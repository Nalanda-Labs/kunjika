// Moderator-only features and the remaining owner-only reads: pinning
// questions, the report queue, editing tag info, and listing your own
// bookmarks. These need a moderator, which only direct database access can
// create, so they live in the harness rather than the Playwright suite.

mod backend_test_common;

use backend_test_common::{login_capture, wait_for_backend_healthy, AuthSession, TestCtx};
use serde_json::{json, Value};

fn moderator(ctx: &TestCtx, prefix: &str) -> AuthSession {
    let s = login_capture(ctx, prefix);
    ctx.exec("UPDATE users SET is_moderator = true WHERE id = $1", &[&s.user_id]);
    s
}

fn create_question(s: &AuthSession, ctx: &TestCtx, tag: &str, title: &str) -> i64 {
    let r = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": title,
            "description": "This description is long enough to pass the twenty-char rule.",
            "tag_list": [tag]}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "create-question");
    let j: Value = r.json().unwrap();
    j["data"]["id"].as_str().unwrap().parse().unwrap()
}

fn answer(s: &AuthSession, ctx: &TestCtx, qid: i64) -> i64 {
    let r = s
        .auth(s.client.post(ctx.url("/answer")))
        .json(&json!({"id": qid, "value": "An answer that is long enough to post."}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "answer");
    let j: Value = r.json().unwrap();
    j["data"].as_str().unwrap().parse().unwrap()
}

fn pin(s: &AuthSession, ctx: &TestCtx, id: i64) -> (u16, Value) {
    let r = s
        .auth(s.client.post(ctx.url(&format!("/bookmark/{id}"))))
        .send()
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().unwrap_or(json!({})))
}

fn is_pinned(ctx: &TestCtx, id: i64) -> bool {
    ctx.q_bool("SELECT coalesce(pinned, false) FROM posts WHERE id = $1", &[&id])
}

#[test]
fn pinning_is_for_moderators_and_limited_to_five_questions() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("pintag");
    let user = login_capture(&ctx, "pinusr");
    let modr = moderator(&ctx, "pinmod");

    // Start from a known state: nothing pinned. Restored at the end.
    let previously: Vec<i64> = ctx
        .db()
        .query("SELECT id FROM posts WHERE pinned = true", &[])
        .unwrap()
        .iter()
        .map(|r| r.get(0))
        .collect();
    ctx.exec0("UPDATE posts SET pinned = false WHERE pinned = true");

    let q = create_question(&user, &ctx, "pintag", "A question to pin to the top?");

    // Regular users cannot pin.
    let (st, _) = pin(&user, &ctx, q);
    assert_eq!(st, 403, "non-moderator pin");
    assert!(!is_pinned(&ctx, q));

    // Missing posts and answers are 404, not a crash.
    assert_eq!(pin(&modr, &ctx, 999_999_999).0, 404, "missing post");
    let a = answer(&user, &ctx, q);
    assert_eq!(pin(&modr, &ctx, a).0, 404, "answers cannot be pinned");

    // Pin and unpin toggle.
    let (st, j) = pin(&modr, &ctx, q);
    assert_eq!((st, j["pinned"].clone()), (200, json!(true)), "pin: {j}");
    assert!(is_pinned(&ctx, q));
    let (st, j) = pin(&modr, &ctx, q);
    assert_eq!((st, j["pinned"].clone()), (200, json!(false)), "unpin: {j}");
    assert!(!is_pinned(&ctx, q));

    // Fill all five slots; a sixth is refused...
    let ids: Vec<i64> = (0..6)
        .map(|i| create_question(&user, &ctx, "pintag", &format!("Pin limit question {i}?")))
        .collect();
    for id in &ids[..5] {
        assert_eq!(pin(&modr, &ctx, *id).0, 200);
    }
    let (st, j) = pin(&modr, &ctx, ids[5]);
    assert_eq!(st, 409, "sixth pin refused: {j}");
    assert!(!is_pinned(&ctx, ids[5]));

    // ...but unpinning still works while full, which frees a slot.
    let (st, j) = pin(&modr, &ctx, ids[0]);
    assert_eq!((st, j["pinned"].clone()), (200, json!(false)), "unpin at limit: {j}");
    assert_eq!(pin(&modr, &ctx, ids[5]).0, 200, "pin after freeing a slot");

    // Pinned questions are listed separately on the front page.
    let front: Value = user
        .client
        .post(ctx.url("/questions/"))
        .json(&json!({"uat": "", "questions_per_page": 10}))
        .send()
        .unwrap()
        .json()
        .unwrap();
    let pinned_ids: Vec<String> = front["pinned"]["questions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_string())
        .collect();
    assert!(pinned_ids.contains(&ids[5].to_string()), "front page pinned list: {front}");

    // Restore.
    ctx.exec0("UPDATE posts SET pinned = false WHERE pinned = true");
    for id in previously {
        ctx.exec("UPDATE posts SET pinned = true WHERE id = $1", &[&id]);
    }
}

#[test]
fn moderators_list_the_report_queue() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("queuetag");
    let author = login_capture(&ctx, "qauth");
    let reporter = login_capture(&ctx, "qrep");
    let modr = moderator(&ctx, "qmod");
    let qid = create_question(&author, &ctx, "queuetag", "A question someone will report?");

    let r = reporter
        .auth(reporter.client.post(ctx.url("/report-post/")))
        .json(&json!({"post_id": qid, "reason": "spam", "details": "queue test"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);

    let list = |s: &AuthSession, body: Value| {
        let r = s
            .auth(s.client.post(ctx.url("/reports/")))
            .json(&body)
            .send()
            .unwrap();
        (r.status().as_u16(), r.json::<Value>().unwrap_or(json!({})))
    };

    // Regular users are refused.
    assert_eq!(list(&reporter, json!({})).0, 403);

    // The moderator sees the new open report.
    let (st, j) = list(&modr, json!({"status": "open", "per_page": 100}));
    assert_eq!(st, 200, "{j}");
    let row = j["data"]["reports"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["post_id"] == json!(qid))
        .cloned()
        .unwrap_or_else(|| panic!("report for {qid} missing: {j}"));
    assert_eq!(row["reason"], json!("spam"));
    assert_eq!(row["status"], json!("open"));
    assert_eq!(row["is_question"], json!(true));

    // An unknown status filter is a 400, and the open-count badge agrees.
    assert_eq!(list(&modr, json!({"status": "bogus"})).0, 400);
    let count: Value = modr
        .auth(modr.client.post(ctx.url("/reports/open-count/")))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert!(count["count"].as_i64().unwrap() >= 1, "open-count: {count}");
}

#[test]
fn moderators_edit_tag_info() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("infotag");
    let modr = moderator(&ctx, "tmod");

    let edit = |info: &str| {
        modr.auth(modr.client.post(ctx.url("/tags/edit/infotag")))
            .json(&json!({"tag_info": info}))
            .send()
            .unwrap()
            .status()
            .as_u16()
    };

    assert_eq!(edit("too short"), 400, "tag info below the 100-char minimum");

    let info = format!("All about infotag. {}", "Useful detail. ".repeat(10));
    assert_eq!(edit(&info), 200);
    let got: Value = modr
        .client
        .get(ctx.url("/get-tag-info/infotag"))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(got["info"], json!(info));

    let missing = modr
        .auth(modr.client.post(ctx.url("/tags/edit/no-such-tag-xyz")))
        .json(&json!({"tag_info": info}))
        .send()
        .unwrap();
    assert_eq!(missing.status().as_u16(), 404, "editing a missing tag");
}

#[test]
fn users_list_their_own_bookmarks() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("bmtag");
    let author = login_capture(&ctx, "bmauth");
    let reader = login_capture(&ctx, "bmread");
    let qid = create_question(&author, &ctx, "bmtag", "A question worth bookmarking?");

    let r = reader
        .auth(reader.client.post(ctx.url(&format!("/bookmark/{qid}/0"))))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "bookmark");

    let r = reader
        .auth(
            reader
                .client
                .post(ctx.url(&format!("/{}/get-bookmarks-by-user/", reader.user_id))),
        )
        .json(&json!({"uat": "", "bookmarks_per_page": 10}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let j: Value = r.json().unwrap();
    assert_eq!(j["count"], json!(1), "one bookmark: {j}");
    assert!(j.to_string().contains("A question worth bookmarking?"), "{j}");
}

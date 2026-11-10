// Community features: deleting posts, closing questions, suspending users,
// edit history, notifications and followed tags, rate limits, feeds and the
// sitemap, and paging through search results.

mod backend_test_common;

use backend_test_common::{login_capture, wait_for_backend_healthy, AuthSession, TestCtx};
use serde_json::{json, Value};

fn moderator(ctx: &TestCtx, prefix: &str) -> AuthSession {
    let s = login_capture(ctx, prefix);
    ctx.exec("UPDATE users SET is_moderator = true WHERE id = $1", &[&s.user_id]);
    s
}

/// POST as `s`; returns (status, JSON body or `{}`).
fn post(s: &AuthSession, ctx: &TestCtx, path: &str, body: Value) -> (u16, Value) {
    let r = s
        .auth(s.client.post(ctx.url(path)))
        .json(&body)
        .send()
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().unwrap_or(json!({})))
}

fn get(ctx: &TestCtx, path: &str) -> (u16, String) {
    let r = ctx.client.get(ctx.url(path)).send().unwrap();
    (r.status().as_u16(), r.text().unwrap_or_default())
}

fn get_json(ctx: &TestCtx, path: &str) -> (u16, Value) {
    let (status, text) = get(ctx, path);
    (status, serde_json::from_str(&text).unwrap_or(json!({})))
}

fn ask(s: &AuthSession, ctx: &TestCtx, tag: &str, title: &str) -> (i64, String) {
    let (st, j) = post(
        s,
        ctx,
        "/create-question",
        json!({
            "title": title,
            "description": "This description is long enough to pass the twenty-char rule.",
            "tag_list": [tag]}),
    );
    assert_eq!(st, 200, "create-question: {j}");
    (
        j["data"]["id"].as_str().unwrap().parse().unwrap(),
        j["data"]["slug"].as_str().unwrap().to_string(),
    )
}

fn answer(s: &AuthSession, ctx: &TestCtx, qid: i64, reply_to: Option<i64>) -> i64 {
    let mut body = json!({"id": qid, "value": "An answer that is long enough to post."});
    if let Some(r) = reply_to {
        body["reply_to"] = json!(r);
    }
    let (st, j) = post(s, ctx, "/answer", body);
    assert_eq!(st, 200, "answer: {j}");
    j["data"].as_str().unwrap().parse().unwrap()
}

fn exists(ctx: &TestCtx, post_id: i64) -> bool {
    ctx.q_i64("SELECT count(*) FROM posts WHERE id = $1", &[&post_id]) == 1
}

#[test]
fn authors_delete_their_own_posts_and_moderators_any() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("deltag");
    let asker = login_capture(&ctx, "delask");
    let answerer = login_capture(&ctx, "delans");
    let modr = moderator(&ctx, "delmod");

    let tag_count =
        || ctx.q_i64("SELECT coalesce(post_count, 0) FROM tags WHERE name = $1::text::citext", &[&"deltag"]);
    let before = tag_count();

    // An unanswered question: its author may delete it, and the tag counter
    // drops back.
    let (lonely, _) = ask(&asker, &ctx, "deltag", "A question nobody will answer?");
    assert_eq!(tag_count(), before + 1);
    assert_eq!(post(&answerer, &ctx, &format!("/delete-post/{lonely}"), json!({})).0, 403);
    let (st, j) = post(&asker, &ctx, &format!("/delete-post/{lonely}"), json!({}));
    assert_eq!(st, 200, "{j}");
    assert!(!exists(&ctx, lonely));
    assert_eq!(tag_count(), before);
    assert_eq!(post(&asker, &ctx, &format!("/delete-post/{lonely}"), json!({})).0, 404);

    // A question with answers.
    let (qid, _) = ask(&asker, &ctx, "deltag", "A question that gets two answers?");
    let a1 = answer(&answerer, &ctx, qid, None);
    let a2 = answer(&answerer, &ctx, qid, None);
    let answer_count = || ctx.q_i64("SELECT answer_count FROM posts WHERE id = $1", &[&qid]);
    assert_eq!(answer_count(), 2);

    // Accept a1, then its author deletes it: the question is no longer
    // marked as answered and has one answer left.
    assert_eq!(post(&asker, &ctx, &format!("/accept-answer/{qid}/{a1}/"), json!({})).0, 200);
    assert!(ctx.q_bool("SELECT answer_accepted FROM posts WHERE id = $1", &[&qid]));
    assert_eq!(post(&asker, &ctx, &format!("/delete-post/{a1}"), json!({})).0, 403, "not the asker's answer");
    let (st, j) = post(&answerer, &ctx, &format!("/delete-post/{a1}"), json!({}));
    assert_eq!(st, 200, "{j}");
    assert_eq!(j["question_id"], json!(qid));
    assert!(!exists(&ctx, a1));
    assert_eq!(answer_count(), 1);
    assert!(!ctx.q_bool("SELECT answer_accepted FROM posts WHERE id = $1", &[&qid]));

    // The asker cannot delete a question that still has an answer...
    let (st, j) = post(&asker, &ctx, &format!("/delete-post/{qid}"), json!({}));
    assert_eq!(st, 409, "{j}");
    assert!(exists(&ctx, qid));

    // ...but a moderator can, and the answer goes with it.
    let (st, j) = post(&modr, &ctx, &format!("/delete-post/{qid}"), json!({}));
    assert_eq!(st, 200, "{j}");
    assert!(!exists(&ctx, qid) && !exists(&ctx, a2));
    assert_eq!(tag_count(), before);
    assert_eq!(
        ctx.q_i64("SELECT count(*) FROM notifications WHERE question_id = $1", &[&qid]),
        0,
        "notifications about a deleted thread are removed"
    );
}

#[test]
fn moderators_close_questions_as_duplicates_and_reopen_them() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("closetag");
    let asker = login_capture(&ctx, "clask");
    let modr = moderator(&ctx, "clmod");
    let (original, original_slug) = ask(&asker, &ctx, "closetag", "The original question on closing?");
    let (dup, dup_slug) = ask(&asker, &ctx, "closetag", "The same question asked again?");
    let a = answer(&asker, &ctx, original, None);

    let question = |id: i64, slug: &str| get_json(&ctx, &format!("/questions/{id}/{slug}")).1["data"].clone();
    assert_eq!(question(dup, &dup_slug)["closed"], json!(false));

    // Only moderators; the target must be a different, existing question.
    let close = format!("/questions/{dup}/close/");
    assert_eq!(post(&asker, &ctx, &close, json!({"duplicate_of": original})).0, 403);
    assert_eq!(post(&modr, &ctx, &close, json!({"duplicate_of": dup})).0, 422);
    assert_eq!(post(&modr, &ctx, &close, json!({"duplicate_of": a})).0, 422, "an answer is not a question");
    assert_eq!(post(&modr, &ctx, &format!("/questions/{a}/close/"), json!({})).0, 404);

    let (st, j) = post(&modr, &ctx, &close, json!({"duplicate_of": original}));
    assert_eq!(st, 200, "{j}");
    let q = question(dup, &dup_slug);
    assert_eq!(q["closed"], json!(true), "{q}");
    assert_eq!(q["duplicate_of"]["id"], json!(original.to_string()));
    assert_eq!(q["duplicate_of"]["slug"], json!(original_slug));

    // A closed question takes no answers; the original still does.
    let (st, j) = post(&asker, &ctx, "/answer", json!({"id": dup, "value": "An answer that is long enough to post."}));
    assert_eq!(st, 409, "{j}");
    answer(&asker, &ctx, original, None);

    // Reopening clears the link and lets answers in again.
    assert_eq!(post(&asker, &ctx, &format!("/questions/{dup}/reopen/"), json!({})).0, 403);
    assert_eq!(post(&modr, &ctx, &format!("/questions/{dup}/reopen/"), json!({})).0, 200);
    let q = question(dup, &dup_slug);
    assert_eq!(q["closed"], json!(false));
    assert!(q["duplicate_of"].is_null(), "{q}");
    answer(&asker, &ctx, dup, None);

    // A plain close has no duplicate; deleting an original unlinks its duplicates.
    assert_eq!(post(&modr, &ctx, &close, json!({})).0, 200);
    assert!(question(dup, &dup_slug)["duplicate_of"].is_null());
    assert_eq!(post(&modr, &ctx, &close, json!({"duplicate_of": original})).0, 200);
    assert_eq!(post(&modr, &ctx, &format!("/delete-post/{original}"), json!({})).0, 200);
    let q = question(dup, &dup_slug);
    assert_eq!(q["closed"], json!(true));
    assert!(q["duplicate_of"].is_null(), "{q}");

    // Answering something that is not a question is a 404, not an orphan row.
    let (st, _) = post(&asker, &ctx, "/answer", json!({"id": 0, "value": "An answer that is long enough to post."}));
    assert_eq!(st, 404);
}

#[test]
fn suspended_users_are_locked_out_until_reinstated() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("susptag");
    let user = login_capture(&ctx, "susus");
    let other = login_capture(&ctx, "susot");
    let modr = moderator(&ctx, "susmod");
    let modr2 = moderator(&ctx, "susmod2");
    let email = ctx.q_string("SELECT email::text FROM users WHERE id = $1", &[&user.user_id]);
    let username = ctx.q_string("SELECT username FROM users WHERE id = $1", &[&user.user_id]);
    let suspend = |by: &AuthSession, target: i64, on: bool| {
        post(by, &ctx, &format!("/users/{target}/suspend/"), json!({"suspended": on}))
    };
    let status = || get_json(&ctx, &format!("/user/{}/{username}", user.user_id)).1["data"]["status"].clone();

    // Who may suspend whom.
    assert_eq!(suspend(&other, user.user_id, true).0, 403, "not a moderator");
    assert_eq!(suspend(&modr, modr.user_id, true).0, 422, "not yourself");
    assert_eq!(suspend(&modr, modr2.user_id, true).0, 403, "moderators cannot suspend moderators");
    assert_eq!(suspend(&modr, 0, true).0, 404);
    assert_eq!(status(), json!("normal"));

    // The user can post before...
    ask(&user, &ctx, "susptag", "A question asked before the suspension?");
    let (st, j) = suspend(&modr, user.user_id, true);
    assert_eq!(st, 200, "{j}");
    assert_eq!(status(), json!("suspended"));

    // ...and afterwards the existing session is refused, as is signing in.
    let (st, j) = post(
        &user,
        &ctx,
        "/create-question",
        json!({"title": "A question asked while suspended?",
               "description": "This description is long enough to pass the twenty-char rule.",
               "tag_list": ["susptag"]}),
    );
    assert_eq!(st, 403, "{j}");
    let r = ctx.client.post(ctx.url("/submit-email")).json(&json!({"email": email})).send().unwrap();
    assert_eq!(r.status().as_u16(), 403, "no sign-in code for a suspended account");

    // Their earlier posts stay, and reinstating restores access.
    assert_eq!(ctx.q_i64("SELECT count(*) FROM posts WHERE posted_by_id = $1", &[&user.user_id]), 1);
    assert_eq!(suspend(&modr, user.user_id, false).0, 200);
    assert_eq!(status(), json!("normal"));
    ask(&user, &ctx, "susptag", "A question asked after being reinstated?");
}

#[test]
fn edits_keep_the_earlier_versions() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("histtag");
    let author = login_capture(&ctx, "hist");
    let (qid, slug) = ask(&author, &ctx, "histtag", "The first title of the question?");
    let aid = answer(&author, &ctx, qid, None);
    let history = |id: i64| get_json(&ctx, &format!("/posts/{id}/history"));
    let edited = || get_json(&ctx, &format!("/questions/{qid}/{slug}")).1["data"]["edited"].clone();

    let (st, h) = history(qid);
    assert_eq!(st, 200, "{h}");
    assert_eq!(h["data"]["revisions"], json!([]));
    assert_eq!(edited(), json!(false));

    // Two edits to the question: each leaves the version it replaced.
    let edit = |title: &str, body: &str| {
        post(&author, &ctx, &format!("/edit-post/{qid}"),
             json!({"title": title, "description": body, "tag_list": ["histtag"]}))
    };
    let first_body = "This description is long enough to pass the twenty-char rule.";
    assert_eq!(edit("The second title of the question?", "The second body, which is long enough.").0, 200);
    assert_eq!(edit("The second title of the question?", "The third body, which is also long enough.").0, 200);
    // Saving without changing anything adds no version.
    assert_eq!(edit("The second title of the question?", "The third body, which is also long enough.").0, 200);

    let (_, h) = history(qid);
    let d = &h["data"];
    assert_eq!(d["is_question"], json!(true));
    assert_eq!(d["title"], json!("The second title of the question?"));
    assert_eq!(d["description"], json!("The third body, which is also long enough."));
    let revs = d["revisions"].as_array().unwrap();
    assert_eq!(revs.len(), 2, "{h}");
    // Newest first.
    assert_eq!(revs[0]["description"], json!("The second body, which is long enough."));
    assert_eq!(revs[1]["title"], json!("The first title of the question?"));
    assert_eq!(revs[1]["description"], json!(first_body));
    assert_eq!(revs[1]["tags"], json!("histtag"));
    assert_eq!(revs[1]["edited_by_id"], json!(author.user_id));
    assert_eq!(edited(), json!(true));

    // Answers have history too, and say so in the answers list.
    let (st, _) = post(&author, &ctx, &format!("/edit/answer/{aid}"),
                       json!({"description": "The answer after it was edited once."}));
    assert_eq!(st, 200);
    let (_, h) = history(aid);
    assert_eq!(h["data"]["is_question"], json!(false));
    assert_eq!(h["data"]["question_id"], json!(qid));
    assert_eq!(h["data"]["revisions"][0]["description"], json!("An answer that is long enough to post."));
    let r = ctx.client.post(ctx.url(&format!("/question/get-answers/{qid}/"))).send().unwrap();
    let answers: Value = r.json().unwrap();
    assert_eq!(answers["data"]["questions"][0]["edited"], json!(true), "{answers}");

    assert_eq!(history(0).0, 404);
}

/// `who`'s notifications as (kind, post_id) pairs, newest first.
fn notifications(ctx: &TestCtx, who: &AuthSession) -> (Vec<(String, i64)>, Value) {
    let (st, j) = post(who, ctx, "/notifications/", json!({"per_page": 100}));
    assert_eq!(st, 200, "{j}");
    let rows = j["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["kind"].as_str().unwrap().to_string(), n["post_id"].as_i64().unwrap()))
        .collect();
    (rows, j)
}

#[test]
fn activity_reaches_the_right_people_as_notifications() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let tag = format!("ntf{}", std::process::id());
    ctx.ensure_tag(&tag);
    let asker = login_capture(&ctx, "ntask");
    let helper = login_capture(&ctx, "nthelp");
    let third = login_capture(&ctx, "ntthird");
    let follower = login_capture(&ctx, "ntfol");
    let unread = |who: &AuthSession| post(who, &ctx, "/notifications/unread-count/", json!({})).1["count"].clone();

    // Following a tag: a new question in it reaches the follower, not the asker.
    assert_eq!(post(&follower, &ctx, "/tags/follow/no-such-tag-here", json!({"following": true})).0, 404);
    let (st, j) = post(&follower, &ctx, &format!("/tags/follow/{tag}"), json!({"following": true}));
    assert_eq!((st, j["following"].clone()), (200, json!(true)));
    // Following twice is harmless.
    assert_eq!(post(&follower, &ctx, &format!("/tags/follow/{tag}"), json!({"following": true})).0, 200);
    assert_eq!(post(&follower, &ctx, "/tags/followed/", json!({})).1["data"], json!([tag]));

    let (qid, _) = ask(&asker, &ctx, &tag, "A question that notifies people?");
    assert_eq!(notifications(&ctx, &follower).0, vec![("tag_question".to_string(), qid)]);
    assert!(notifications(&ctx, &asker).0.is_empty(), "nobody is notified of their own action");

    // helper answers, replying to the asker -> the asker gets a `reply`.
    let a1 = answer(&helper, &ctx, qid, Some(asker.user_id));
    assert_eq!(notifications(&ctx, &asker).0, vec![("reply".to_string(), a1)]);
    assert!(notifications(&ctx, &helper).0.is_empty());

    // third answers without replying to anyone -> both participants get an `answer`.
    let a2 = answer(&third, &ctx, qid, None);
    assert_eq!(notifications(&ctx, &asker).0[0], ("answer".to_string(), a2));
    assert_eq!(notifications(&ctx, &helper).0, vec![("answer".to_string(), a2)]);
    assert!(notifications(&ctx, &third).0.is_empty());

    // Accepting helper's answer and upvoting it both reach helper. Voting
    // down and up again does not add a second unread upvote.
    assert_eq!(post(&asker, &ctx, &format!("/accept-answer/{qid}/{a1}/"), json!({})).0, 200);
    for vote in [1, -1, 1] {
        assert_eq!(post(&asker, &ctx, "/votes", json!({"vote": vote, "id": a1})).0, 200);
    }
    let (rows, j) = notifications(&ctx, &helper);
    assert_eq!(
        rows,
        vec![("upvote".to_string(), a1), ("accepted".to_string(), a1), ("answer".to_string(), a2)],
        "{j}"
    );
    // Each row carries what the page needs to render and link it.
    let first = &j["data"][0];
    assert_eq!(first["question_id"], json!(qid));
    assert_eq!(first["question_title"], json!("A question that notifies people?"));
    assert_eq!(first["actor_id"], json!(asker.user_id));
    assert_eq!(first["read"], json!(false));
    assert_eq!(j["unread"], json!(3));
    assert_eq!(unread(&helper), json!(3));

    // Cursor pagination: one at a time, same order, then the end.
    let mut cursor: Option<String> = None;
    let mut walked = Vec::new();
    loop {
        let mut body = json!({"per_page": 1});
        if let Some(c) = &cursor {
            body["cursor"] = json!(c);
        }
        let (st, page) = post(&helper, &ctx, "/notifications/", body);
        assert_eq!(st, 200, "{page}");
        walked.push(page["data"][0]["kind"].as_str().unwrap().to_string());
        match page["next_cursor"].as_str() {
            Some(c) => cursor = Some(c.to_string()),
            None => break,
        }
    }
    assert_eq!(walked, ["upvote", "accepted", "answer"]);
    assert_eq!(post(&helper, &ctx, "/notifications/", json!({"cursor": "nonsense"})).0, 400);

    // Marking one read, then all. Another user's notification is untouched.
    let one = j["data"][0]["id"].as_i64().unwrap();
    let (st, r) = post(&helper, &ctx, "/notifications/read/", json!({"id": one}));
    assert_eq!((st, r["unread"].clone()), (200, json!(2)));
    assert_eq!(post(&asker, &ctx, "/notifications/read/", json!({"id": one})).0, 200);
    assert_eq!(unread(&helper), json!(2), "only the owner can mark a notification read");
    assert_eq!(post(&helper, &ctx, "/notifications/read/", json!({})).1["unread"], json!(0));
    assert_eq!(unread(&asker), json!(2), "mark-all touched only helper's notifications");

    // Unfollowing stops tag notifications.
    let (_, j) = post(&follower, &ctx, &format!("/tags/follow/{tag}"), json!({"following": false}));
    assert_eq!(j["following"], json!(false));
    ask(&asker, &ctx, &tag, "A second question after the unfollow?");
    assert_eq!(notifications(&ctx, &follower).0.len(), 1);

    // The email opt-out is per user and off by request only.
    let settings = |body: Value| post(&helper, &ctx, "/notifications/settings/", body).1["email_notifications"].clone();
    assert_eq!(settings(json!({})), json!(true));
    assert_eq!(settings(json!({"email_notifications": false})), json!(false));
    assert_eq!(settings(json!({})), json!(false));
    assert!(!ctx.q_bool("SELECT email_notifications FROM users WHERE id = $1", &[&helper.user_id]));
    assert!(ctx.q_bool("SELECT email_notifications FROM users WHERE id = $1", &[&asker.user_id]));

    // Anonymous callers get nothing.
    let r = ctx.client.post(ctx.url("/notifications/")).json(&json!({})).send().unwrap();
    assert!(matches!(r.status().as_u16(), 400 | 401));
}

#[test]
fn posting_is_rate_limited_per_user() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("ratetag");
    let asker = login_capture(&ctx, "rtask");
    let eager = login_capture(&ctx, "rteager");
    let (qid, _) = ask(&asker, &ctx, "ratetag", "A question that gets far too many answers?");

    // The default limit is 30 answers an hour: the 31st is refused with a
    // Retry-After, and nothing is stored for it.
    for _ in 0..30 {
        answer(&eager, &ctx, qid, None);
    }
    let r = eager
        .auth(eager.client.post(ctx.url("/answer")))
        .json(&json!({"id": qid, "value": "An answer that is long enough to post."}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 429);
    let retry: u64 = r.headers()["retry-after"].to_str().unwrap().parse().unwrap();
    assert!((1..=3600).contains(&retry), "retry-after {retry}");
    assert_eq!(ctx.q_i64("SELECT count(*) FROM posts WHERE op_id = $1", &[&qid]), 30);

    // The limit is per user: someone else can still answer.
    answer(&asker, &ctx, qid, None);

    // Questions: 10 an hour.
    for n in 0..10 {
        ask(&eager, &ctx, "ratetag", &format!("Rate limited question number {n}?"));
    }
    let (st, j) = post(
        &eager,
        &ctx,
        "/create-question",
        json!({"title": "The eleventh question in an hour?",
               "description": "This description is long enough to pass the twenty-char rule.",
               "tag_list": ["ratetag"]}),
    );
    assert_eq!(st, 429, "{j}");

    // Sign-in codes: 5 an hour per address, counted whether or not the
    // address belongs to an account.
    let email = format!("ratelimit{}@example.com", std::process::id());
    let mut last = 0;
    for _ in 0..6 {
        last = ctx.client.post(ctx.url("/submit-email")).json(&json!({"email": email})).send().unwrap().status().as_u16();
    }
    assert_eq!(last, 429);
}

#[test]
fn feeds_sitemap_and_search_paging() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let tag = format!("feed{}", std::process::id());
    ctx.ensure_tag(&tag);
    let author = login_capture(&ctx, "feed");
    // A word no other question contains, so the search results are ours alone.
    let word = format!("zyxfeed{}", std::process::id());
    let mut ids = Vec::new();
    for n in 1..=3 {
        ids.push(ask(&author, &ctx, &tag, &format!("Is {word} <b> & number {n} escaped?")));
    }
    let (newest, newest_slug) = ids[2].clone();

    // Tag feed: our three questions, newest first, markup escaped.
    let r = ctx.client.get(ctx.url(&format!("/feeds/tagged/{tag}/"))).send().unwrap();
    assert_eq!(r.status().as_u16(), 200);
    assert!(r.headers()["content-type"].to_str().unwrap().starts_with("application/rss+xml"));
    let xml = r.text().unwrap();
    assert_eq!(xml.matches("<item>").count(), 3, "{xml}");
    assert!(xml.contains(&format!("Is {word} &lt;b&gt; &amp; number 3 escaped?")), "{xml}");
    assert!(!xml.contains("<b>"), "raw markup must not leak into the feed");
    assert!(xml.contains(&format!("/questions/{newest}/{newest_slug}</link>")), "{xml}");
    let pos = |n: usize| xml.find(&format!("number {n} escaped")).unwrap();
    assert!(pos(3) < pos(2) && pos(2) < pos(1), "newest first");

    // An unknown tag is an empty, valid feed; the main feed has our newest.
    let (st, xml) = get(&ctx, "/feeds/tagged/no-such-tag-anywhere/");
    assert_eq!(st, 200);
    assert!(xml.contains("<channel>") && !xml.contains("<item>"));
    let (st, xml) = get(&ctx, "/feeds/questions/");
    assert_eq!(st, 200);
    assert!(xml.contains(&format!("/questions/{newest}/{newest_slug}")));

    // Sitemap and robots.
    let (st, xml) = get(&ctx, "/sitemap.xml");
    assert_eq!(st, 200);
    assert!(xml.contains("<urlset") && xml.contains(&format!("/questions/{newest}/{newest_slug}</loc>")));
    let (st, robots) = get(&ctx, "/robots.txt");
    assert_eq!(st, 200);
    assert!(robots.contains("/sitemap.xml"), "{robots}");

    // Search: the total is reported, and offset pages through it.
    let search = |per_page: i64, offset: Option<i64>| {
        let mut body = json!({"search": word, "questions_per_page": per_page});
        if let Some(o) = offset {
            body["offset"] = json!(o);
        }
        let r = ctx.client.post(ctx.url("/search-questions/")).json(&body).send().unwrap();
        assert_eq!(r.status().as_u16(), 200);
        let j: Value = r.json().unwrap();
        let ids: Vec<String> = j["data"]["questions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|q| q["id"].as_str().unwrap().to_string())
            .collect();
        (ids, j["count"].as_i64().unwrap())
    };
    let (all, count) = search(50, None);
    assert_eq!((all.len(), count), (3, 3));
    let (first, count) = search(2, Some(0));
    let (second, _) = search(2, Some(2));
    let (beyond, count_beyond) = search(2, Some(10));
    assert_eq!(count, 3, "count is the total, not the page size");
    assert_eq!([first, second].concat(), all, "pages line up with the full list");
    assert!(beyond.is_empty());
    assert_eq!(count_beyond, 3);
    assert_eq!(search(2, Some(-5)).0, all[..2], "a negative offset is the first page");
}

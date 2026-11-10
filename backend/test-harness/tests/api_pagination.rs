// Pagination contracts:
//
//   * a question's answers are paged with `?limit=&offset=` and come with a
//     total `count` (plus `?answer={id}` to find the page holding one answer);
//   * the moderation queue is paged with an opaque keyset `cursor` /
//     `next_cursor`, which must stay correct while reports are being filed.
//
// Expected orders are read straight from the database, so the tests also hold
// against a queue that already contains reports from earlier runs.

mod backend_test_common;

use backend_test_common::{login_capture, wait_for_backend_healthy, AuthSession, TestCtx};
use serde_json::{json, Value};

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

fn answer(s: &AuthSession, ctx: &TestCtx, qid: i64, n: usize) -> i64 {
    let r = s
        .auth(s.client.post(ctx.url("/answer")))
        .json(&json!({"id": qid, "value": format!("Answer number {n}, long enough to post.")}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "answer");
    let j: Value = r.json().unwrap();
    j["data"].as_str().unwrap().parse().unwrap()
}

fn ids(ctx: &TestCtx, sql: &str, param: &(dyn postgres::types::ToSql + Sync)) -> Vec<i64> {
    ctx.db()
        .query(sql, &[param])
        .unwrap()
        .iter()
        .map(|r| r.get::<_, i64>(0))
        .collect()
}

/// `GET`-style read of one answers page: (status, answer ids, body).
fn answers_page(ctx: &TestCtx, qid: i64, query: &str) -> (u16, Vec<i64>, Value) {
    let r = ctx
        .client
        .post(ctx.url(&format!("/question/get-answers/{qid}/?{query}")))
        .send()
        .unwrap();
    let status = r.status().as_u16();
    let j: Value = r.json().unwrap_or(json!({}));
    let rows = j["data"]["questions"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|x| x["question_id"].as_str().unwrap().parse().unwrap())
                .collect()
        })
        .unwrap_or_default();
    (status, rows, j)
}

#[test]
fn answers_are_paged_with_limit_and_offset() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("pagetag");
    let author = login_capture(&ctx, "pgauth");
    let answerer = login_capture(&ctx, "pgans");
    let qid = create_question(&author, &ctx, "pagetag", "A question with five answers to page?");
    for n in 1..=5 {
        answer(&answerer, &ctx, qid, n);
    }
    let expected = ids(
        &ctx,
        "SELECT id FROM posts WHERE op_id = $1 ORDER BY created_at ASC, id ASC",
        &qid,
    );
    assert_eq!(expected.len(), 5);

    // Three pages of two: 2 + 2 + 1, in order, each reporting the total.
    for (offset, want) in [(0, &expected[0..2]), (2, &expected[2..4]), (4, &expected[4..5])] {
        let (st, got, j) = answers_page(&ctx, qid, &format!("limit=2&offset={offset}"));
        assert_eq!(st, 200, "{j}");
        assert_eq!(got, want, "offset {offset}: {j}");
        assert_eq!(j["count"], json!(5), "{j}");
        assert_eq!(j["offset"], json!(offset), "{j}");
    }

    // Past the end: an empty page that still knows the total.
    let (st, got, j) = answers_page(&ctx, qid, "limit=2&offset=6");
    assert_eq!(st, 200, "{j}");
    assert!(got.is_empty(), "{j}");
    assert_eq!(j["count"], json!(5), "{j}");

    // Both parameters are optional, and out-of-range values are clamped
    // instead of reaching the database (`limit -1` used to be a 500).
    for query in ["", "limit=-1&offset=-7", "limit=0", "limit=100000"] {
        let (st, got, j) = answers_page(&ctx, qid, query);
        assert_eq!(st, 200, "`{query}`: {j}");
        assert_eq!(got.first(), expected.first(), "`{query}`: {j}");
        assert_eq!(j["offset"], json!(0), "`{query}`: {j}");
    }
    let (_, got, _) = answers_page(&ctx, qid, "limit=-1");
    assert_eq!(got.len(), 1, "limit below 1 clamps to 1");
    let (_, got, _) = answers_page(&ctx, qid, "");
    assert_eq!(got, expected, "default limit returns them all");

    // `answer=` finds the page that holds one answer.
    let (st, got, j) = answers_page(&ctx, qid, &format!("limit=2&answer={}", expected[4]));
    assert_eq!(st, 200, "{j}");
    assert_eq!(j["offset"], json!(4), "{j}");
    assert_eq!(got, &expected[4..5], "{j}");
    let (_, got, j) = answers_page(&ctx, qid, &format!("limit=2&answer={}", expected[3]));
    assert_eq!(j["offset"], json!(2), "{j}");
    assert_eq!(got, &expected[2..4], "{j}");
    // An id that is not an answer to this question falls back to page one.
    let (st, got, j) = answers_page(&ctx, qid, &format!("limit=2&answer={qid}"));
    assert_eq!(st, 200, "{j}");
    assert_eq!(j["offset"], json!(0), "{j}");
    assert_eq!(got, &expected[0..2], "{j}");

    // A question without answers: empty page, count 0.
    let empty = create_question(&author, &ctx, "pagetag", "A question nobody has answered yet?");
    let (st, got, j) = answers_page(&ctx, empty, "limit=2&offset=0");
    assert_eq!(st, 200, "{j}");
    assert!(got.is_empty(), "{j}");
    assert_eq!(j["count"], json!(0), "{j}");
}

/// One page of the open queue: (status, report ids, next_cursor, body).
fn reports_page(
    ctx: &TestCtx,
    modr: &AuthSession,
    per_page: i64,
    cursor: Option<&str>,
) -> (u16, Vec<i64>, Option<String>, Value) {
    let mut body = json!({"status": "open", "per_page": per_page});
    if let Some(c) = cursor {
        body["cursor"] = json!(c);
    }
    let r = modr
        .auth(modr.client.post(ctx.url("/reports/")))
        .json(&body)
        .send()
        .unwrap();
    let status = r.status().as_u16();
    let j: Value = r.json().unwrap_or(json!({}));
    let rows = j["data"]["reports"]
        .as_array()
        .map(|a| a.iter().map(|x| x["id"].as_i64().unwrap()).collect())
        .unwrap_or_default();
    let next = j["next_cursor"].as_str().map(str::to_string);
    (status, rows, next, j)
}

fn report(s: &AuthSession, ctx: &TestCtx, post_id: i64) {
    let r = s
        .auth(s.client.post(ctx.url("/report-post/")))
        .json(&json!({"post_id": post_id, "reason": "spam", "details": "pagination test"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "report-post");
}

#[test]
fn reports_are_paged_with_a_stable_cursor() {
    const PER_PAGE: i64 = 4;
    const OPEN_ORDER: &str = "SELECT id FROM post_reports WHERE status = $1 \
                              ORDER BY created_at DESC, id DESC";

    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    ctx.ensure_tag("cursortag");
    let author = login_capture(&ctx, "crauth");
    let reporter = login_capture(&ctx, "crrep");
    let late_reporter = login_capture(&ctx, "crlate");
    let modr = login_capture(&ctx, "crmod");
    ctx.exec("UPDATE users SET is_moderator = true WHERE id = $1", &[&modr.user_id]);

    // Five fresh open reports (the per-user daily limit), on five questions.
    let mut qids = Vec::new();
    for n in 1..=5 {
        let qid = create_question(&author, &ctx, "cursortag", &format!("Reported question {n}?"));
        report(&reporter, &ctx, qid);
        qids.push(qid);
    }
    let expected = ids(&ctx, OPEN_ORDER, &"open");
    assert!(expected.len() as i64 > PER_PAGE, "need more than one page");

    // First page: the newest reports, a cursor for the rest, the full count.
    let (st, first, cursor, j) = reports_page(&ctx, &modr, PER_PAGE, None);
    assert_eq!(st, 200, "{j}");
    assert_eq!(first, &expected[..PER_PAGE as usize], "{j}");
    assert_eq!(j["count"], json!(expected.len()), "{j}");
    assert_eq!(j["data"]["next_cursor"], j["next_cursor"], "{j}");
    let mut cursor = cursor.unwrap_or_else(|| panic!("first page must have a next_cursor: {j}"));

    // A report filed between two page requests lands at the head of the
    // queue. With an offset that would shift every row down one and repeat
    // the last row of page one; the cursor must carry on exactly where it was.
    report(&late_reporter, &ctx, qids[0]);

    let mut walked = first;
    for _ in 0..1000 {
        let (st, page, next, j) = reports_page(&ctx, &modr, PER_PAGE, Some(&cursor));
        assert_eq!(st, 200, "{j}");
        assert!(!page.is_empty(), "a cursor never leads to an empty page: {j}");
        walked.extend(&page);
        match next {
            Some(n) => {
                assert_eq!(page.len() as i64, PER_PAGE, "only the last page may be short: {j}");
                cursor = n;
            }
            None => break,
        }
    }
    assert_eq!(walked, expected, "every report once, in order, unaffected by the new one");

    // Starting over picks the late report up as the newest row.
    let (_, fresh, _, j) = reports_page(&ctx, &modr, PER_PAGE, None);
    assert_eq!(fresh, &ids(&ctx, OPEN_ORDER, &"open")[..PER_PAGE as usize], "{j}");
    assert!(!expected.contains(&fresh[0]), "{j}");

    // A page that holds the whole queue has no next_cursor.
    let total = expected.len() as i64 + 1;
    if total <= 100 {
        let (st, all, next, j) = reports_page(&ctx, &modr, total, None);
        assert_eq!(st, 200);
        assert_eq!(all.len() as i64, total);
        assert!(next.is_none(), "exactly-full last page must not advertise more: {j}");
    }

    // An empty cursor is the first page; a malformed one is a 400, not a 500.
    let (st, again, _, _) = reports_page(&ctx, &modr, PER_PAGE, Some(""));
    assert_eq!((st, again), (200, fresh));
    for bad in ["garbage", "12_", "_3", "1_2_3", "99999999999999999999999999999999999999999_1"] {
        let (st, _, _, j) = reports_page(&ctx, &modr, PER_PAGE, Some(bad));
        assert_eq!(st, 400, "cursor `{bad}`: {j}");
    }
}

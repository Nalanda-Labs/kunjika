// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// Post-reports API tests, part 2: rate limit (5/day), actioned review
// soft-deletes the post, and auth requirements.
// ADDITIVE-ONLY: lives in backend/test-harness/, never touches
// backend/src/* or backend/Cargo.toml.

mod backend_test_common;

use backend_test_common::{login_capture, wait_for_backend_healthy, AuthSession, TestCtx};
use serde_json::json;

fn create_question(author: &AuthSession, ctx: &TestCtx, tag: &str, title: &str) -> i64 {
    let q: serde_json::Value = author
        .auth(author.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": title,
            "description": "This description is long enough to pass the twenty-char rule.",
            "tag_list": [tag]}))
        .send()
        .unwrap()
        .json()
        .unwrap_or(serde_json::json!({}));
    q.get("data")
        .and_then(|d| d.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .parse::<i64>()
        .expect("create-question data.id numeric")
}

#[test]
fn report_rate_limit_and_actioned_deletes_post() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let reporter = login_capture(&ctx, "rrate");
    let author = login_capture(&ctx, "rvict");
    let moderator = login_capture(&ctx, "rexec");
    ctx.exec(
        "UPDATE users SET is_superuser = true WHERE id = $1",
        &[&moderator.user_id],
    );
    ctx.ensure_tag("rrltag");
    let mut last_status = 0;
    for i in 0..6 {
        let qid = create_question(&author, &ctx, "rrltag", &format!("Rate limit probe {i}?"));
        let r = reporter
            .auth(reporter.client.post(ctx.url("/report-post/")))
            .json(&json!({"post_id": qid, "reason": "other"}))
            .send()
            .unwrap();
        last_status = r.status().as_u16();
        if i < 5 {
            assert_eq!(last_status, 200, "report {i} within limit must be 200");
        }
    }
    assert_eq!(last_status, 429, "6th report in 24h must be 429, got {last_status}");
    let doomed = create_question(&author, &ctx, "rrltag", "Why will this post be deleted?");
    let witness = login_capture(&ctx, "rwitn");
    let r = witness
        .auth(witness.client.post(ctx.url("/report-post/")))
        .json(&json!({"post_id": doomed, "reason": "abusive"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "witness report 200");
    let report_id: i64 = ctx.q_i64(
        "SELECT id FROM post_reports WHERE post_id = $1 AND status = 'open' ORDER BY id DESC LIMIT 1",
        &[&doomed],
    );
    let r = moderator
        .auth(moderator.client.post(ctx.url(&format!("/reports/{report_id}/review/"))))
        .json(&json!({"status": "actioned"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "actioned 200");
    let remaining: i64 =
        ctx.q_i64("SELECT count(*) FROM posts WHERE id = $1", &[&doomed]);
    assert_eq!(remaining, 0, "actioned must hard-delete post {doomed}");
    let status: String =
        ctx.q_string("SELECT status FROM post_reports WHERE id = $1", &[&report_id]);
    assert_eq!(status, "actioned", "report row marked actioned");
}

#[test]
fn report_endpoints_require_auth() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let r = ctx
        .client
        .post(ctx.url("/report-post/"))
        .json(&json!({"post_id": 1, "reason": "spam"}))
        .send()
        .expect("anon report-post failed");
    assert!(
        r.status().as_u16() == 400 || r.status().as_u16() == 401,
        "anon report-post 400/401, got {}",
        r.status()
    );
    let r = ctx
        .client
        .post(ctx.url("/reports/"))
        .json(&json!({"status": "open", "per_page": 5}))
        .send()
        .expect("anon reports listing failed");
    assert!(
        r.status().as_u16() == 400 || r.status().as_u16() == 401,
        "anon reports 400/401, got {}",
        r.status()
    );
}

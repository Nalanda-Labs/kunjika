// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// Post-reports API tests, part 1: file/validate/duplicate/self-report,
// reviewer gating, and the review lifecycle.
// ADDITIVE-ONLY: lives in backend/test-harness/, never touches
// backend/src/* or backend/Cargo.toml.

mod backend_test_common;

use backend_test_common::{login_capture, wait_for_backend_healthy, AuthSession, TestCtx};
use serde_json::json;

/// Create a question as `author` and return its numeric post id.
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
fn report_lifecycle_and_review() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let reporter = login_capture(&ctx, "rprep");
    let author = login_capture(&ctx, "rauth");
    let moderator = login_capture(&ctx, "rmodx");
    ctx.exec(
        "UPDATE users SET is_moderator = true WHERE id = $1",
        &[&moderator.user_id],
    );
    ctx.ensure_tag("rreptag");
    let qid = create_question(&author, &ctx, "rreptag", "Why is this post reportable?");
    let r = reporter
        .auth(reporter.client.post(ctx.url("/report-post/")))
        .json(&json!({"post_id": qid, "reason": "spam", "details": "Looks like spam."}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "report-post 200");
    let j: serde_json::Value = r.json().unwrap();
    assert_eq!(j.get("success"), Some(&json!(true)), "created, got {j}");
    let r = reporter
        .auth(reporter.client.post(ctx.url("/report-post/")))
        .json(&json!({"post_id": qid, "reason": "abusive"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "duplicate returns 200 with success=false");
    let j: serde_json::Value = r.json().unwrap();
    assert_eq!(j.get("success"), Some(&serde_json::json!(false)), "duplicate flagged, got {j}");
    let r = author
        .auth(author.client.post(ctx.url("/report-post/")))
        .json(&json!({"post_id": qid, "reason": "spam"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 400, "self-report must be 400");
    let other = create_question(&author, &ctx, "rreptag", "Why is this other post bad?");
    let r = reporter
        .auth(reporter.client.post(ctx.url("/report-post/")))
        .json(&json!({"post_id": other, "reason": "i-dont-like-it"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 400, "bad reason must be 400");
    let r = reporter
        .auth(reporter.client.post(ctx.url("/reports/")))
        .json(&json!({"status": "open", "per_page": 50}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 403, "non-mod queue must be 403");
    let r = moderator
        .auth(moderator.client.post(ctx.url("/reports/")))
        .json(&json!({"status": "open", "per_page": 50}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "mod queue 200");
    let j: serde_json::Value = r.json().unwrap();
    let reports = j
        .get("data")
        .and_then(|d| d.get("reports"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let entry = reports
        .iter()
        .find(|rep| rep.get("post_id").and_then(|v| v.as_i64()) == Some(qid))
        .unwrap_or_else(|| panic!("queue must contain post {qid}, got {j}"));
    let report_id = entry.get("id").and_then(|v| v.as_i64()).expect("report id");
    assert_eq!(
        entry.get("reason").and_then(|v| v.as_str()),
        Some("spam"),
        "reason round-trips"
    );
    let r = moderator
        .auth(moderator.client.post(ctx.url("/reports/open-count/")))
        .json(&json!({}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "open-count 200");
    let j: serde_json::Value = r.json().unwrap();
    assert!(
        j.get("count").and_then(|v| v.as_i64()).unwrap_or(0) >= 1,
        "open-count >= 1, got {j}"
    );
    let r = reporter
        .auth(reporter.client.post(ctx.url(&format!("/reports/{report_id}/review/"))))
        .json(&json!({"status": "dismissed"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 403, "non-mod review must be 403");
    let r = moderator
        .auth(moderator.client.post(ctx.url(&format!("/reports/{report_id}/review/"))))
        .json(&json!({"status": "dismissed"}))
        .send()
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "dismiss 200");
    let j: serde_json::Value = r.json().unwrap();
    assert_eq!(j.get("success"), Some(&json!(true)), "dismiss ok, got {j}");
    let j: serde_json::Value = moderator
        .auth(moderator.client.post(ctx.url("/reports/")))
        .json(&json!({"status": "open", "per_page": 50}))
        .send()
        .unwrap()
        .json()
        .unwrap();
    let open: Vec<i64> = j
        .get("data")
        .and_then(|d| d.get("reports"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|rep| rep.get("id").and_then(|v| v.as_i64())).collect())
        .unwrap_or_default();
    assert!(!open.contains(&report_id), "dismissed leaves open queue");
}

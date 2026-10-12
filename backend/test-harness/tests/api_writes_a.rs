// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// Authenticated write-path tests, part 1: question lifecycle.
// ADDITIVE-ONLY: lives in backend/test-harness/, never touches
// backend/src/* or backend/Cargo.toml.

mod backend_test_common;

use backend_test_common::{login_capture, wait_for_backend_healthy, TestCtx};
use serde_json::json;

#[test]
fn full_question_lifecycle() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let s = login_capture(&ctx, "qlife");
    ctx.ensure_tag("qtesttag");

    // QuestionRequest: title 6..256, description 20..100000, tag_list required.
    let q: serde_json::Value = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "How do integration tests work here?",
            "description": "This description is long enough to pass the twenty-char rule.",
            "tag_list": ["qtesttag"]}))
        .send()
        .unwrap()
        .json()
        .unwrap_or(serde_json::json!({}));
    let qid = q
        .get("data")
        .and_then(|d| d.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!qid.is_empty(), "create-question data.id, got {q}");
    let qid_i: i64 = qid.parse().expect("qid numeric");

    let slug = ctx.q_string("SELECT slug FROM posts WHERE id = $1", &[&qid_i]);
    let g = s
        .client
        .get(ctx.url(&format!("/questions/{qid_i}/{slug}")))
        .send()
        .unwrap();
    assert!(
        g.status().is_success(),
        "get question 2xx, got {}",
        g.status()
    );

    // `r.tag_list.len() > 5` -> 422 "Too many tags."
    let bad = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Too many tags here ok?",
            "description": "This description is long enough to pass validation indeed.",
            "tag_list": ["a", "b", "c", "d", "e", "f"]}))
        .send()
        .unwrap();
    assert_eq!(
        bad.status().as_u16(),
        422,
        "6 tags must be 422, got {}: {}",
        bad.status(),
        bad.text().unwrap_or_default()
    );

    // Empty tag list -> 422 "All tags were not found".
    let none = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question with no tags at all?",
            "description": "This description is long enough to pass validation indeed.",
            "tag_list": []}))
        .send()
        .unwrap();
    assert_eq!(
        none.status().as_u16(),
        422,
        "0 tags must be 422, got {}",
        none.status()
    );
}

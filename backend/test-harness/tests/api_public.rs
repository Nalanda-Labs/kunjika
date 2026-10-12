// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// ADDITIVE-ONLY: lives in backend/test-harness/, never touches
// backend/src/* or backend/Cargo.toml.

mod backend_test_common;

use backend_test_common::{wait_for_backend_healthy, TestCtx};
use serde_json::json;

#[test]
fn public_read_endpoints_respond() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let r = ctx
        .client
        .post(ctx.url("/tags"))
        .json(&json!({"tag": "", "direction": null}))
        .send()
        .expect("POST /tags failed");
    assert!(r.status().is_success(), "POST /tags 2xx");
    let r = ctx
        .client
        .post(ctx.url("/search-tags"))
        .json(&json!({"tag": "a", "tags_per_page": 5}))
        .send()
        .expect("POST /search-tags failed");
    assert!(r.status().is_success(), "POST /search-tags 2xx");
    let r = ctx
        .client
        .post(ctx.url("/questions/"))
        .json(&json!({"uat": "", "questions_per_page": 5, "direction": null}))
        .send()
        .expect("POST /questions/ failed");
    assert!(r.status().is_success(), "POST /questions/ 2xx");
    for path in ["/get-questions-count/", "/get-unanswered-questions-count/"] {
        let r = ctx
            .client
            .get(ctx.url(path))
            .send()
            .unwrap_or_else(|_| panic!("GET {path} failed"));
        assert!(r.status().is_success(), "GET {path} 2xx");
    }
    let r = ctx
        .client
        .post(ctx.url("/users"))
        .json(&json!({"last_user": "", "direction": null}))
        .send()
        .expect("POST /users failed");
    assert!(r.status().is_success(), "POST /users 2xx");
}

#[test]
fn protected_writes_reject_anonymous_calls() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let r = ctx
        .client
        .post(ctx.url("/create-question"))
        .json(&json!({
            "title": "Why is the sky blue?",
            "description": "This description is long enough to pass validation rules here.",
            "tag_list": ["physics"]}))
        .send()
        .expect("anon create-question failed");
    assert!(
        r.status().as_u16() == 400 || r.status().as_u16() == 401,
        "anon create-question 400/401, got {}",
        r.status()
    );
    let r = ctx
        .client
        .post(ctx.url("/votes"))
        .json(&json!({"vote": 1, "id": 1}))
        .send()
        .expect("anon vote failed");
    assert!(
        r.status().as_u16() == 400 || r.status().as_u16() == 401,
        "anon vote 400/401, got {}",
        r.status()
    );
}

// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// ADDITIVE-ONLY: lives in backend/test-harness/, never touches
// backend/src/* or backend/Cargo.toml.

mod backend_test_common;

use backend_test_common::wait_for_backend_healthy;
use backend_test_common::{unique_email, unique_username, TestCtx};
use serde_json::json;

#[test]
fn register_rejects_invalid_payloads() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let cases: Vec<(serde_json::Value, &str)> = vec![
        (
            json!({"username": "ab", "email": unique_email("bad1")}),
            "short username",
        ),
        (
            json!({"username": "1badname", "email": unique_email("bad2")}),
            "digit username",
        ),
        (
            json!({"username": "bad@name", "email": unique_email("bad3")}),
            "@ username",
        ),
        (
            json!({"username": unique_username("badmail"), "email": "not-an-email"}),
            "bad email",
        ),
    ];
    for (body, label) in cases {
        let r = ctx
            .client
            .post(ctx.url("/register"))
            .json(&body)
            .send()
            .expect("register request failed");
        assert_eq!(r.status().as_u16(), 400, "{label} must be 400");
    }
    // plus-addressed email banned by Register::check_email
    let plus = format!("plus+{}@example.com", unique_username("p"));
    let r = ctx
        .client
        .post(ctx.url("/register"))
        .json(&json!({"username": unique_username("plus"), "email": plus}))
        .send()
        .expect("plus email request failed");
    assert_eq!(r.status().as_u16(), 400, "+ email must be 400");
}

#[test]
fn check_username_availability_roundtrip() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let username = unique_username("avail");
    let email = unique_email("avail");
    let r = ctx
        .client
        .post(ctx.url("/check-username-availability"))
        .json(&json!({"username": username}))
        .send()
        .expect("availability free failed");
    assert_eq!(r.status().as_u16(), 200);
    let reg = ctx
        .client
        .post(ctx.url("/register"))
        .json(&json!({"username": username, "email": email}))
        .send()
        .expect("register for availability failed");
    assert_eq!(reg.status().as_u16(), 200);
    let r2 = ctx
        .client
        .post(ctx.url("/check-username-availability"))
        .json(&json!({"username": username}))
        .send()
        .expect("availability taken failed");
    assert_eq!(r2.status().as_u16(), 200);
    let body: serde_json::Value = r2.json().expect("availability body JSON");
    let taken = body
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
        == false
        || body.to_string().to_lowercase().contains("unavail");
    assert!(taken, "taken username must be flagged: {body}");
}

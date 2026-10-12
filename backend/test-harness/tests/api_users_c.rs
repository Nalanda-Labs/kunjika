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
fn submit_email_for_unknown_user_is_rejected() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let r = ctx
        .client
        .post(ctx.url("/submit-email"))
        .json(&json!({"email": unique_email("ghost")}))
        .send()
        .expect("submit-email ghost failed");
    assert!(!r.status().is_success(), "unknown email must not be 2xx");
}

#[test]
fn otp_login_roundtrip_with_db_code() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let username = unique_username("otp");
    let email = unique_email("otp");
    let reg = ctx
        .client
        .post(ctx.url("/register"))
        .json(&json!({"username": username, "email": email}))
        .send()
        .expect("register for otp failed");
    assert_eq!(reg.status().as_u16(), 200);
    ctx.exec(
        "UPDATE users SET email_verified = true WHERE email = $1",
        &[&email],
    );
    let sub = ctx
        .client
        .post(ctx.url("/submit-email"))
        .json(&json!({"email": email}))
        .send()
        .expect("submit-email failed");
    assert_eq!(sub.status().as_u16(), 200);
    let otp = ctx.q_string(
        "SELECT token FROM tokens WHERE email = $1 ORDER BY created_date DESC LIMIT 1",
        &[&email],
    );
    let bad = ctx
        .client
        .post(ctx.url("/auth/login"))
        .json(&json!({"email": email, "otp": "0000000000"}))
        .send()
        .expect("bad otp failed");
    assert_eq!(bad.status().as_u16(), 401, "wrong OTP must be 401");
    let good = ctx
        .client
        .post(ctx.url("/auth/login"))
        .json(&json!({"email": email, "otp": otp}))
        .send()
        .expect("good otp failed");
    assert_eq!(good.status().as_u16(), 200, "correct OTP must be 200");
    let cookies: Vec<String> = good
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        cookies.iter().any(|c| c.contains("access_token")),
        "login must set access_token, got: {cookies:?}"
    );
}

// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// Mail-flow integration tests: register -> confirmation token -> confirm-email,
// and submit-email -> OTP token -> auth/login. Mail delivery itself is asserted
// via the Mailpit HTTP API; tokens are also read back from YSQL as the
// authoritative source.
//
// ADDITIVE-ONLY: lives in backend/test-harness/, never touches
// backend/src/* or backend/Cargo.toml.
//
// Env (see backend_test_common.rs): KUNJIKA_TEST_BASE_URL,
// KUNJIKA_TEST_DB_URL, KUNJIKA_TEST_MAILPIT_URL.

mod backend_test_common;

use backend_test_common::wait_for_backend_healthy;
use backend_test_common::{unique_email, unique_username, TestCtx};
use serde_json::{json, Value};

fn mailpit_messages(ctx: &TestCtx) -> Vec<Value> {
    let r = ctx
        .client
        .get(format!(
            "{}/api/v1/messages?limit=50",
            ctx.mailpit_url.trim_end_matches('/')
        ))
        .send()
        .expect("mailpit messages failed");

    assert!(r.status().is_success(), "mailpit API must be 2xx");
    let v: Value = r.json().expect("mailpit messages JSON");

    v.get("messages")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default()
}

fn mailpit_latest_for(messages: &[Value], to: &str) -> Option<Value> {
    messages
        .iter()
        .rev()
        .find(|m| {
            let s = m.to_string().to_lowercase();
            s.contains(&to.to_lowercase())
        })
        .cloned()
}

/// Fetch a message's plain-text body.
///
/// NOTE: `/view/latest.html` is useless here - these mails have no HTML part
/// and Mailpit replies "This message does not contain a HTML part". The v1
/// JSON API exposes the decoded body under `Text`.
fn mailpit_body(ctx: &TestCtx, id: &str) -> String {
    let r = ctx
        .client
        .get(format!(
            "{}/api/v1/message/{}",
            ctx.mailpit_url.trim_end_matches('/'),
            id
        ))
        .send()
        .expect("mailpit body failed");

    assert!(r.status().is_success(), "mailpit message {id} must exist");

    let v: Value = r.json().expect("mailpit message JSON");
    [
        v.get("Text").and_then(|t| t.as_str()).unwrap_or(""),
        v.get("HTML").and_then(|t| t.as_str()).unwrap_or(""),
    ]
    .concat()
}

#[test]
fn register_sends_confirmation_email_with_token_link() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let username = unique_username("mailreg");
    let email = unique_email("mailreg");

    let reg = ctx
        .client
        .post(ctx.url("/register"))
        .json(&json!({"username": username, "email": email}))
        .send()
        .expect("register failed");
    assert_eq!(
        reg.status().as_u16(),
        200,
        "register must be 200, body: {}",
        reg.text().unwrap_or_default()
    );

    // Token row exists in YSQL ...
    let token = ctx.q_string(
        "SELECT token FROM tokens WHERE email = $1 ORDER BY created_date DESC LIMIT 1",
        &[&email],
    );

    assert!(!token.is_empty());

    // ... and the mail actually arrived in Mailpit. Poll rather than a fixed
    // sleep: when the harness tests run in parallel, several `register` calls
    // fire SMTP sends at once and the confirmation mail can land a moment after
    // a 2s window, which made this test flaky.
    let msg = (0..30)
        .find_map(|_| {
            let messages = mailpit_messages(&ctx);
            mailpit_latest_for(&messages, &email)
        })
        .unwrap_or_else(|| panic!("no mailpit message for {email} after polling"));
    let id = msg.get("ID").and_then(|v| v.as_str()).unwrap_or("");
    let body = mailpit_body(&ctx, id);

    assert!(
        body.to_lowercase().contains("confirm"),
        "confirmation mail should mention confirm, got {body:.200}"
    );
}

#[test]
fn confirm_email_verifies_user() {
    let ctx = TestCtx::new();

    wait_for_backend_healthy(&ctx, 60);

    let username = unique_username("confirm");
    let email = unique_email("confirm");

    let reg = ctx
        .client
        .post(ctx.url("/register"))
        .json(&json!({"username": username, "email": email}))
        .send()
        .expect("register failed");

    assert_eq!(reg.status().as_u16(), 200);

    let token = ctx.q_string(
        "SELECT token FROM tokens WHERE email = $1 ORDER BY created_date DESC LIMIT 1",
        &[&email],
    );

    let r = ctx
        .client
        .get(ctx.url(&format!("/confirm-email/{token}")))
        .send()
        .expect("confirm-email failed");
    assert!(
        r.status().is_success(),
        "confirm-email must be 2xx, got {}",
        r.status()
    );

    let verified = ctx.q_bool(
        "SELECT email_verified FROM users WHERE email = $1",
        &[&email],
    );

    assert!(verified, "user must be verified after confirm-email");
}

#[test]
fn submit_email_sends_otp_and_login_consumes_it() {
    let ctx = TestCtx::new();

    wait_for_backend_healthy(&ctx, 60);

    let username = unique_username("otpmail");
    let email = unique_email("otpmail");

    let reg = ctx
        .client
        .post(ctx.url("/register"))
        .json(&json!({"username": username, "email": email}))
        .send()
        .expect("register failed");
    assert_eq!(reg.status().as_u16(), 200);
    ctx.exec(
        "UPDATE users SET email_verified = true WHERE email = $1",
        &[&email],
    );

    // NOTE: do not DELETE /api/v1/messages here. The harness tests share one
    // Mailpit inbox and run in parallel, so a global wipe destroys messages
    // other tests are polling for. The OTP is read from YSQL below (unique per
    // email), so we don't need a clean inbox to assert single-use semantics.
    let sub = ctx
        .client
        .post(ctx.url("/submit-email"))
        .json(&json!({"email": email}))
        .send()
        .expect("submit-email failed");

    assert_eq!(
        sub.status().as_u16(),
        200,
        "submit-email must be 200, body: {}",
        sub.text().unwrap_or_default()
    );

    let otp = ctx.q_string(
        "SELECT token FROM tokens WHERE email = $1 ORDER BY created_date DESC LIMIT 1",
        &[&email],
    );
    assert_eq!(otp.len(), 10, "login_code_length=10, got {otp:?}");

    // Poll for THIS test's OTP mail (parallel tests share the inbox, so match
    // the exact recipient rather than checking the whole mailbox).
    let messages = (0..30)
        .find_map(|_| {
            let m = mailpit_messages(&ctx);
            if mailpit_latest_for(&m, &email).is_some() {
                Some(m)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("expected OTP mail for {email} in mailpit after polling"));

    // OTP is single-use: first login 200, replay must fail.
    let good = ctx
        .client
        .post(ctx.url("/auth/login"))
        .json(&json!({"email": email, "otp": otp}))
        .send()
        .expect("otp login failed");

    assert_eq!(good.status().as_u16(), 200, "correct OTP must be 200");

    let replay = ctx
        .client
        .post(ctx.url("/auth/login"))
        .json(&json!({"email": email, "otp": otp}))
        .send()
        .expect("otp replay failed");

    assert!(
        !replay.status().is_success(),
        "replayed OTP must not be 2xx"
    );
}

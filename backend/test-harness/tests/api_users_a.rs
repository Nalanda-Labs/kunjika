// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// ADDITIVE-ONLY: lives in backend/test-harness/, never touches
// backend/src/* or backend/Cargo.toml.

mod backend_test_common;

use backend_test_common::{login_capture, mailpit_message_count, wait_for_backend_healthy};
use backend_test_common::{unique_email, unique_username, TestCtx};
use serde_json::json;

#[test]
fn register_happy_path_creates_user_row() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let username = unique_username("regok");
    let email = unique_email("regok");
    let before = mailpit_message_count(&ctx);
    let resp = ctx
        .client
        .post(ctx.url("/register"))
        .json(&json!({"username": username, "email": email}))
        .send()
        .expect("POST /register failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "register should succeed, body: {}",
        resp.text().unwrap_or_default()
    );
    let found = ctx.q_string("SELECT username FROM users WHERE email = $1", &[&email]);
    assert_eq!(found, username);
    let verified = ctx.q_bool(
        "SELECT email_verified FROM users WHERE email = $1",
        &[&email],
    );
    assert!(!verified);
    let tokens = ctx.q_i64("SELECT COUNT(*) FROM tokens WHERE email = $1", &[&email]);
    assert!(tokens >= 1, "expected a confirmation token row");
    let after = mailpit_message_count(&ctx);
    if after > before {
        assert!(after >= before + 1);
    }
}

#[test]
fn summary_works_for_user_without_posts() {
    // BEFORE: users::dao::get_summary ran `fetch_one` against the GROUP-BY
    // views `answers_count` / `questions_count_by_user`, which only contain
    // rows for users who have actually posted. A freshly registered user
    // produced RowNotFound and `GET /{id}/summary` answered 500 with
    // "no rows returned by a query that expected to return at least one
    // row" - breaking the Summary tab of every new user's profile page.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);

    let s = login_capture(&ctx, "summary");
    let resp = ctx
        .client
        .get(ctx.url(&format!("/{}/summary", s.user_id)))
        .send()
        .expect("GET /{id}/summary failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "summary must work for a user with no answers/questions, got: {}",
        resp.text().unwrap_or_default()
    );
    let body: serde_json::Value = resp.json().expect("summary response is JSON");
    assert_eq!(body["data"]["answers_count"], json!(0), "{body}");
    assert_eq!(body["data"]["questions_count"], json!(0), "{body}");
    assert!(body["data"]["karma"].is_number(), "{body}");
}

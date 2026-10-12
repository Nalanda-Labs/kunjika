// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// Regression tests for the username length mismatch that used to exist between
// the API validator and the schema.
//
// BEFORE: `Register` in backend/src/users/user.rs validates
//     #[validate(length(min = 3, max = 33), custom(function = "validate_username"))]
//     pub username: String,
// but migrations/20220620050241_initial.sql declared
//     username character varying(10) NOT NULL
// An 11..33 character username therefore passed validation and then failed the
// INSERT with SQLSTATE 22001 ("value too long for type character varying(10)"),
// and backend/src/users/routes.rs::register maps *any* `user_add` error to
//     HTTP 409 {"status":"fail","message":"Either email or username is taken"}
// i.e. a length/DB error was reported to the user as a name collision.
//
// FIXED BY: migrations/20261001160000_widen_username_to_33.sql, which widens the
// column to `character varying(33)` - now matching the validator.
//
// These tests pin the fixed behaviour at both boundaries.
//
// ADDITIVE-ONLY: lives in backend/test-harness/.

mod backend_test_common;

use backend_test_common::{unique_email, username_of_len, wait_for_backend_healthy, TestCtx};
use serde_json::json;

fn register(ctx: &TestCtx, username: &str, email: &str) -> reqwest::StatusCode {
    ctx.client
        .post(ctx.url("/register"))
        .json(&json!({"username": username, "email": email}))
        .send()
        .expect("register request")
        .status()
}

#[test]
fn register_username_of_11_chars_succeeds() {
    // This is the value that used to be rejected with a misleading 409.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let username = username_of_len(11);
    assert_eq!(username.len(), 11);
    let email = unique_email("len11");

    assert_eq!(
        register(&ctx, &username, &email),
        200u16,
        "11 chars is inside the validator range (3..=33) and now fits the column"
    );

    // ... and the row really is persisted this time.
    let stored = ctx.q_string("SELECT username FROM users WHERE email = $1", &[&email]);
    assert_eq!(stored, username);
    assert!(
        !ctx.q_bool(
            "SELECT email_verified FROM users WHERE email = $1",
            &[&email]
        ),
        "a freshly registered user starts unverified"
    );
}

#[test]
fn register_username_of_exactly_33_chars_succeeds() {
    // Widest value the validator AND the widened column both accept.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let username = username_of_len(33);
    assert_eq!(username.len(), 33);
    let email = unique_email("len33");

    assert_eq!(
        register(&ctx, &username, &email),
        200u16,
        "exactly 33 chars is the documented upper bound and must be accepted"
    );
    let stored = ctx.q_string("SELECT username FROM users WHERE email = $1", &[&email]);
    assert_eq!(stored, username);
}

#[test]
fn register_rejects_username_over_33_chars() {
    // One past the bound: rejected by the validator with a 400.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let email = unique_email("len34");
    assert_eq!(
        register(&ctx, &username_of_len(34), &email),
        400u16,
        "34 chars must be a 400"
    );
    let count = ctx.q_i64("SELECT count(*) FROM users WHERE email = $1", &[&email]);
    assert_eq!(
        count, 0,
        "nothing must be persisted for an invalid username"
    );
}

#[test]
fn register_username_of_10_chars_still_succeeds() {
    // Non-boundary sanity check: the old limit must keep working.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let username = username_of_len(10);
    assert_eq!(username.len(), 10);
    let email = unique_email("len10");

    assert_eq!(register(&ctx, &username, &email), 200u16);
    let stored = ctx.q_string("SELECT username FROM users WHERE email = $1", &[&email]);
    assert_eq!(stored, username);
}

#[test]
fn duplicate_username_still_reports_409_taken() {
    // The 409 "taken" message must still be produced by an actual collision -
    // i.e. it no longer masks length errors, but it still does its real job.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let username = username_of_len(12);
    let first = unique_email("dupeA");
    assert_eq!(register(&ctx, &username, &first), 200u16);

    let second = unique_email("dupeB");
    assert_eq!(
        register(&ctx, &username, &second),
        409u16,
        "a genuine collision must still be 409"
    );
    // The second (duplicate) registration must not have created a row.
    let count = ctx.q_i64("SELECT count(*) FROM users WHERE email = $1", &[&second]);
    assert_eq!(count, 0);
}

#[test]
fn register_maps_non_unique_db_error_to_500_not_409() {
    // `register` used to translate *any* `user_add` failure into
    // 409 "Either email or username is taken". Now only SQLSTATE 23505 maps to
    // 409; anything else is a 500 and is logged.
    //
    // A temporary CHECK constraint gives us a non-unique failure mode
    // (SQLSTATE 23514) that is otherwise hard to trigger through the API,
    // because the validator rejects every input the column would reject.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);

    let username = username_of_len(9);
    let constraint = format!("chk_{username}");
    ctx.exec0(&format!(
        "ALTER TABLE users ADD CONSTRAINT {constraint} CHECK (username <> '{username}')"
    ));

    let email = unique_email("checkviol");
    let status = register(&ctx, &username, &email);

    // Drop the constraint before asserting so a failure cannot leave it behind.
    ctx.exec0(&format!(
        "ALTER TABLE users DROP CONSTRAINT IF EXISTS {constraint}"
    ));

    assert_eq!(
        status.as_u16(),
        500,
        "a CHECK violation is not a name collision and must not answer 409"
    );
    let count = ctx.q_i64("SELECT count(*) FROM users WHERE email = $1", &[&email]);
    assert_eq!(count, 0, "the failed INSERT must not persist a row");
}

#[test]
fn duplicate_email_case_insensitively_reports_409_taken() {
    // `users.email` is `citext`, so "Case@Ex.com" and "case@ex.com" are the same
    // address. `20220620050241_initial.sql` declares
    //     ADD CONSTRAINT users_email_key UNIQUE (email);
    // but YugabyteDB cannot index a citext column, so that constraint silently
    // never applied. migrations/20261001160000_widen_username_to_33.sql
    // restores it as a unique index over `lower(email)`.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let email = unique_email("casedupe");

    assert_eq!(
        register(&ctx, &username_of_len(10), &email),
        200u16,
        "first registration must succeed"
    );

    // Same address, different casing -> must be rejected as a conflict.
    let flipped = email.replace("example.com", "Example.COM");
    assert_ne!(flipped, email, "the flipped address must differ textually");

    let second = username_of_len(10);
    assert_eq!(
        register(&ctx, &second, &flipped),
        409u16,
        "citext uniqueness must reject a differently-cased duplicate email"
    );
}

#[test]
fn duplicate_tag_names_are_rejected_by_the_database() {
    // Same story for `tags.name` (citext): the intended
    // `tags_name_key UNIQUE (name)` never applied on YugabyteDB, which is what
    // let duplicate tag rows accumulate and made `insert_question` fail with
    // the misleading "All tags were not found".
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);

    let tag = format!("dup{}", username_of_len(6));
    ctx.exec("INSERT INTO tags (name) VALUES ($1)", &[&tag]);

    // A differently-cased duplicate must now be refused by the index.
    let upper = tag.to_uppercase();
    assert_ne!(upper, tag);
    assert!(
        !ctx.try_exec("INSERT INTO tags (name) VALUES ($1)", &[&upper]),
        "a case-differing duplicate tag name must be rejected by tags_name_key"
    );
    assert_eq!(
        ctx.q_i64(
            "SELECT count(*) FROM tags WHERE lower(name) = lower($1)",
            &[&tag]
        ),
        1,
        "exactly one row per tag name"
    );
}

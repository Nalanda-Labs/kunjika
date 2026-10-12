// Copyright (c) 2026 Kunjika test harness contributors.
// SPDX-License-Identifier: GPL-3.0-only
//
// Authenticated write-path tests, part 2: answer + edit + vote + bookmark.
// ADDITIVE-ONLY: lives in backend/test-harness/, never touches
// backend/src/* or backend/Cargo.toml.

mod backend_test_common;

use backend_test_common::{login_capture, unique_email, wait_for_backend_healthy, TestCtx};
use serde_json::json;

#[test]
fn answer_edit_vote_bookmark() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let s = login_capture(&ctx, "lifecycle");
    ctx.ensure_tag("lifetag");
    let q: serde_json::Value = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Lifecycle question title here?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["lifetag"]}))
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
    assert!(!qid.is_empty(), "create-question qid, got {q}");
    let qid_i: i64 = qid.parse().expect("qid numeric");

    // AnswerReq: `id` is the question id as a String, `value` >= 20 chars.
    //
    // The answer must come from a DIFFERENT user: the handler builds a
    // notification email addressed to `find_participants` results and calls
    // `.body(...).unwrap()`, which panics with `MissingTo` when there is no
    // one else involved in the question (see
    // `answer_to_own_question_panics_missing_to`).
    //
    // `reply_to` must parse as i64 or `questions::dao::insert_answer` panics
    // on `.unwrap()`. Note the column is `posts.reply_to_id`, whose FK points
    // at **users(id)** (not posts), and the SvelteKit client sends the post
    // author's user id - so that is what we mirror here.
    let answerer = login_capture(&ctx, "answerer");
    let ans: serde_json::Value = answerer
        .auth(answerer.client.post(ctx.url("/answer")))
        .json(&json!({"id": qid,
            "value": "Here is an answer certainly longer than twenty chars.",
            "reply_to": s.user_id.to_string()}))
        .send()
        .unwrap()
        .json()
        .unwrap_or(serde_json::json!({}));
    let aid = ans
        .get("data")
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!aid.is_empty(), "answer data id, got {ans}");
    let aid_i: i64 = aid.parse().expect("aid numeric");

    // EditRequest: title/description/tag_list.
    let edit = s
        .auth(s.client.post(ctx.url(&format!("/edit-post/{qid_i}"))))
        .json(&json!({"title": "Lifecycle question title here? (edited)",
            "description": "Edited description still longer than twenty characters.",
            "tag_list": ["lifetag"]}))
        .send()
        .unwrap();
    assert!(
        edit.status().is_success(),
        "edit 2xx, got {}: {}",
        edit.status(),
        edit.text().unwrap_or_default()
    );

    // A second user votes: first ok, replay rejected as VoteOnce.
    let v = login_capture(&ctx, "voter");
    let v1: serde_json::Value = v
        .auth(v.client.post(ctx.url("/votes")))
        .json(&json!({"vote": 1, "id": qid_i}))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(
        v1.get("success").and_then(|x| x.as_bool()),
        Some(true),
        "vote1 {v1}"
    );
    let v2: serde_json::Value = v
        .auth(v.client.post(ctx.url("/votes")))
        .json(&json!({"vote": 1, "id": qid_i}))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert!(
        v2.to_string().contains("only once"),
        "vote replay rejected, got {v2}"
    );

    // Bookmark needs BOTH ids: POST /bookmark/{qid}/{aid}.
    let b = v
        .auth(
            v.client
                .post(ctx.url(&format!("/bookmark/{qid_i}/{aid_i}"))),
        )
        .send()
        .unwrap();
    assert!(
        b.status().is_success(),
        "bookmark 2xx, got {}: {}",
        b.status(),
        b.text().unwrap_or_default()
    );
}

#[test]
fn answer_accepts_every_reply_to_shape() {
    // BEFORE: questions::dao::insert_answer did
    //     answer.reply_to.parse::<i64>().unwrap()
    // so `"reply_to": ""` panicked the ntex worker with
    // `ParseIntError { kind: Empty }` and aborted the connection with no
    // status line - a remote crash reachable by any logged-in user.
    //
    // The SvelteKit client is also inconsistent about the field: it sends
    // `response.data.posted_by_id` as a JSON *number*, and omits the key
    // entirely for a top-level answer (`reply_to_id` starts `undefined`), which
    // serde renders as a missing field. Both used to fail deserialisation.
    //
    // `AnswerReq` now deserialises every shape into `Option<i64>` and binds it
    // straight through, so an absent reply target is a NULL `reply_to_id`.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);

    let author = login_capture(&ctx, "shapea");
    ctx.ensure_tag("shapetag");
    let q: serde_json::Value = author
        .auth(author.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question used for reply_to shape tests?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["shapetag"]}))
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
    assert!(!qid.is_empty(), "create-question qid, got {q}");

    // A second user answers, so the notification email has a recipient and the
    // request does not run into the separate MissingTo panic.
    let answerer = login_capture(&ctx, "shapeb");
    let value = "This answer value is definitely longer than twenty chars.";

    let cases: Vec<(&str, serde_json::Value)> = vec![
        ("omitted", json!({"id": qid, "value": value})),
        ("null", json!({"id": qid, "value": value, "reply_to": null})),
        (
            "empty string",
            json!({"id": qid, "value": value, "reply_to": ""}),
        ),
        (
            "whitespace",
            json!({"id": qid, "value": value, "reply_to": "   "}),
        ),
        (
            "json number",
            json!({"id": qid, "value": value, "reply_to": author.user_id}),
        ),
        (
            "numeric string",
            json!({"id": qid, "value": value, "reply_to": author.user_id.to_string()}),
        ),
        // `id` arrives from the route as a string, but tolerate a number too.
        (
            "numeric id",
            json!({"id": qid.parse::<i64>().unwrap(), "value": value, "reply_to": null}),
        ),
    ];

    let mut empty_shape_answer: Option<String> = None;
    for (label, body) in cases {
        let r = answerer
            .auth(answerer.client.post(ctx.url("/answer")))
            .json(&body)
            .send()
            .unwrap_or_else(|e| panic!("{label}: request failed outright: {e}"));
        let status = r.status().as_u16();
        let text = r.text().unwrap_or_default();
        assert_eq!(status, 200, "{label}: expected 200, got {status}: {text}");
        if label == "empty string" {
            empty_shape_answer = Some(text.clone());
        }
    }

    // An absent reply target must be stored as NULL, not a bogus 0.
    let raw = empty_shape_answer.expect("empty-string case ran");
    let body: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("answer response not JSON: {raw}"));
    let aid = body
        .get("data")
        .and_then(|d| d.as_str())
        .expect("answer id");
    let aid_i: i64 = aid
        .parse()
        .unwrap_or_else(|e| panic!("answer id not numeric: {aid}: {e}"));
    assert!(
        ctx.q_opt_i64("SELECT reply_to_id FROM posts WHERE id = $1", &[&aid_i])
            .is_none(),
        "reply_to_id must be NULL when the client sends no reply target"
    );

    // And the server is still healthy - the old code died here.
    let health = ctx.client.get(ctx.url("/get-questions-count/")).send();
    assert!(
        health.map(|r| r.status().is_success()).unwrap_or(false),
        "server must survive the panic and keep serving"
    );
}

#[test]
fn get_answers_returns_rows_for_plain_answers() {
    // BEFORE: `questions::AR` declared `rusername`/`rimage_url` as plain
    // `String`, but the DAO selects them via
    //     left join users as u1 on t.reply_to_id = u1.id
    // so both are NULL for every answer that is not a reply. sqlx refused to
    // decode that ("decoding column 12: unexpected null; try decoding as an
    // `Option`") and the endpoint answered 500 for ANY question whose answers
    // included at least one ordinary (non-reply) answer - i.e. every real
    // question. `answer_accepts_every_reply_to_shape` only asserted DB state,
    // so the read path was never exercised against NULL reply rows.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);

    let author = login_capture(&ctx, "ansfetch");
    ctx.ensure_tag("fetchtag");
    let q: serde_json::Value = author
        .auth(author.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question used for the get-answers decode test?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["fetchtag"]}))
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
    assert!(!qid.is_empty(), "create-question qid, got {q}");
    let qid_i: i64 = qid.parse().expect("qid numeric");

    // A different user posts one PLAIN answer (no reply_to -> NULL reply
    // columns) and one reply (non-NULL reply columns), so the result set
    // exercises both decode paths. A different author avoids the separate
    // MissingTo panic in the answer notification mail.
    let answerer = login_capture(&ctx, "ansfetch2");
    let bodies = [
        json!({"id": qid, "value": "A plain answer longer than twenty characters."}),
        json!({"id": qid, "value": "A reply answer longer than twenty characters!",
               "reply_to": author.user_id}),
    ];
    for body in bodies {
        let r = answerer
            .auth(answerer.client.post(ctx.url("/answer")))
            .json(&body)
            .send()
            .unwrap();
        let status = r.status().as_u16();
        let text = r.text().unwrap_or_default();
        assert_eq!(status, 200, "answer 200, got {status}: {text}");
    }

    // The SvelteKit client sends the question's asked_ts as `cat`; any
    // timestamp before the answers works as the pagination cursor, so use a
    // fixed past date and expect both rows back.
    let r = author
        .auth(
            author
                .client
                .post(ctx.url(&format!("/question/get-answers/{qid_i}/?limit=50"))),
        )
        .json(&json!({"cat": "2020-01-01T00:00:00Z"}))
        .send()
        .unwrap();
    let status = r.status().as_u16();
    let text = r.text().unwrap_or_default();
    assert_eq!(status, 200, "get-answers 200, got {status}: {text}");

    let body: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("not JSON: {text}: {e}"));
    let rows = body["data"]["questions"]
        .as_array()
        .unwrap_or_else(|| panic!("data.questions array, got {body}"));
    assert_eq!(rows.len(), 2, "both answers returned, got {body}");
    assert!(
        rows.iter().any(|a| a["reply_to_id"].is_null()
            && a["rusername"].is_null()
            && a["rimage_url"].is_null()),
        "plain answer must decode with null reply columns, got {body}"
    );
    assert!(
        rows.iter().any(|a| a["reply_to_id"] == json!(author.user_id)
            && a["rusername"].is_string()),
        "reply must carry the replied-to username, got {body}"
    );
}

#[test]
fn edit_post_requires_ownership() {
    // BEFORE: the handler took `_auth: AuthorizationService` and never used it,
    // so ANY logged-in user could rewrite ANY post's title, body and tags.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);

    let owner = login_capture(&ctx, "editown");
    ctx.ensure_tag("edittag");
    let q: serde_json::Value = owner
        .auth(owner.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question owned by the first user?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["edittag"]}))
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
    assert!(!qid.is_empty(), "create-question qid, got {q}");
    let qid_i: i64 = qid.parse().expect("qid numeric");

    let intruder = login_capture(&ctx, "editbad");

    let r = intruder
        .auth(
            intruder
                .client
                .post(ctx.url(&format!("/edit-post/{qid_i}"))),
        )
        .json(&json!({
            "title": "HIJACKED BY A DIFFERENT USER",
            "description": "This body was written by somebody else entirely.",
            "tag_list": ["edittag"]}))
        .send()
        .unwrap();
    assert_eq!(
        r.status().as_u16(),
        403,
        "editing another user's post must be 403, got {}: {}",
        r.status(),
        r.text().unwrap_or_default()
    );

    // Nothing may have changed.
    assert_eq!(
        ctx.q_string("SELECT title FROM posts WHERE id = $1", &[&qid_i]),
        "Question owned by the first user?",
        "the post must be untouched after a refused edit"
    );
    assert_eq!(
        ctx.q_string("SELECT description FROM posts WHERE id = $1", &[&qid_i]),
        "This description is long enough to pass validation rules."
    );

    // The author themself may edit.
    let ok = owner
        .auth(owner.client.post(ctx.url(&format!("/edit-post/{qid_i}"))))
        .json(&json!({
            "title": "Question owned by the first user, edited?",
            "description": "The author rewrote this description on purpose.",
            "tag_list": ["edittag"]}))
        .send()
        .unwrap();
    assert_eq!(
        ok.status().as_u16(),
        200,
        "the author must be able to edit, got {}: {}",
        ok.status(),
        ok.text().unwrap_or_default()
    );
    assert_eq!(
        ctx.q_string("SELECT title FROM posts WHERE id = $1", &[&qid_i]),
        "Question owned by the first user, edited?"
    );
}

#[test]
fn edit_answer_requires_ownership() {
    // The same rule applies to answers, which share the endpoint.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);

    let author = login_capture(&ctx, "ansown");
    let other = login_capture(&ctx, "ansoth");
    ctx.ensure_tag("anstag");

    let q: serde_json::Value = author
        .auth(author.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question that will receive an answer?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["anstag"]}))
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
    let qid_i: i64 = qid.parse().expect("qid numeric");
    assert!(!qid.is_empty(), "create-question qid, got {q}");

    let ans: serde_json::Value = author
        .auth(author.client.post(ctx.url("/answer")))
        .json(&json!({"id": qid,
            "value": "An answer written by the question author here.",
            "reply_to": author.user_id}))
        .send()
        .unwrap()
        .json()
        .unwrap_or(serde_json::json!({}));
    let aid = ans
        .get("data")
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!aid.is_empty(), "answer data id, got {ans}");
    let aid_i: i64 = aid.parse().expect("aid numeric");

    let r = other
        .auth(other.client.post(ctx.url(&format!("/edit-post/{aid_i}"))))
        .json(&json!({"description": "Rewritten by a user with no rights here."}))
        .send()
        .unwrap();
    assert_eq!(
        r.status().as_u16(),
        403,
        "editing another user's answer must be 403, got {}",
        r.status()
    );
    assert_eq!(
        ctx.q_string("SELECT description FROM posts WHERE id = $1", &[&aid_i]),
        "An answer written by the question author here.",
        "the answer must be untouched"
    );
    let _ = qid_i;
}

#[test]
fn edit_post_validates_and_rejects_duplicate_tags() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let s = login_capture(&ctx, "editval");
    ctx.ensure_tag("edvalpha");

    let q: serde_json::Value = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question used for edit validation?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["edvalpha"]}))
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
    let qid_i: i64 = qid.parse().expect("qid numeric");
    let url = format!("/edit-post/{qid_i}");

    let short = s
        .auth(s.client.post(ctx.url(&url)))
        .json(&json!({"title": "abc",
            "description": "This description is long enough to pass validation.",
            "tag_list": ["edvalpha"]}))
        .send()
        .unwrap();
    assert_eq!(
        short.status().as_u16(),
        400,
        "a 3-character title must be 400, got {}: {}",
        short.status(),
        short.text().unwrap_or_default()
    );

    let tiny = s
        .auth(s.client.post(ctx.url(&url)))
        .json(&json!({"title": "A perfectly fine title here",
            "description": "too short", "tag_list": ["edvalpha"]}))
        .send()
        .unwrap();
    assert_eq!(
        tiny.status().as_u16(),
        400,
        "a 9-character body must be 400, got {}",
        tiny.status()
    );

    let dup = s
        .auth(s.client.post(ctx.url(&url)))
        .json(&json!({"title": "A perfectly fine title here",
            "description": "This description is long enough to pass validation.",
            "tag_list": ["edvalpha", "EDVALPHA"]}))
        .send()
        .unwrap();
    assert_eq!(
        dup.status().as_u16(),
        422,
        "a duplicate tag must be 422, got {}: {}",
        dup.status(),
        dup.text().unwrap_or_default()
    );

    let missing = s
        .auth(s.client.post(ctx.url("/edit-post/99999999")))
        .json(&json!({"title": "A perfectly fine title here",
            "description": "This description is long enough to pass validation.",
            "tag_list": ["edvalpha"]}))
        .send()
        .unwrap();
    assert_eq!(
        missing.status().as_u16(),
        404,
        "editing a non-existent post must be 404, got {}",
        missing.status()
    );
}

#[test]
fn inserting_a_token_prunes_expired_tokens() {
    // BEFORE: `tokens_delete_old_rows()` was a copy-paste of
    // `views_delete_old_rows()` and deleted from `views` instead, so expired
    // tokens never expired. On YugabyteDB the `views` DELETE additionally
    // raised 40001 "schema version mismatch" from inside the trigger and
    // aborted the enclosing INSERT. Fixed by
    // migrations/20261001161000_fix_tokens_delete_old_rows.sql.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);

    let stale = unique_email("stale");
    let fresh = unique_email("fresh");
    ctx.exec(
        "INSERT INTO tokens (email, token) VALUES ($1, 'stale-token')",
        &[&stale],
    );
    ctx.exec(
        "UPDATE tokens SET created_date = now() - interval '30 minutes' WHERE email = $1",
        &[&stale],
    );
    assert_eq!(
        ctx.q_i64("SELECT count(*) FROM tokens WHERE email = $1", &[&stale]),
        1,
        "fixture must start with the stale token present"
    );

    // Any new token insert fires the AFTER INSERT trigger.
    ctx.exec(
        "INSERT INTO tokens (email, token) VALUES ($1, 'fresh-token')",
        &[&fresh],
    );

    assert_eq!(
        ctx.q_i64("SELECT count(*) FROM tokens WHERE email = $1", &[&stale]),
        0,
        "the expired token must be pruned from `tokens`, not from `views`"
    );
    assert_eq!(
        ctx.q_i64("SELECT count(*) FROM tokens WHERE email = $1", &[&fresh]),
        1,
        "the fresh token must survive"
    );
}

#[test]
fn register_still_succeeds_with_the_fixed_trigger() {
    // The broken trigger made `register` (and `submit-email`) fail on
    // YugabyteDB, because the INSERT ran a DELETE against `views`. This is the
    // regression guard for the end-to-end path.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let s = login_capture(&ctx, "trigfix");
    let _ = s;
    assert!(
        true,
        "login_capture would have panicked on a 500 from register"
    );
}

#[test]
fn duplicate_tags_are_rejected() {
    // BEFORE: a repeated tag reached `insert_question`, whose
    // `tag_ids.len() != tag_list.len()` guard fired and reported the
    // misleading "All tags were not found" even though every tag existed.
    // Now the route rejects it up front with 422 and names the offender.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let s = login_capture(&ctx, "duptags");
    ctx.ensure_tag("dupalpha");
    ctx.ensure_tag("dupbeta");

    // Exact duplicate.
    let r = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question with a repeated tag here?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["dupalpha", "dupalpha"]}))
        .send()
        .unwrap();
    assert_eq!(
        r.status().as_u16(),
        422,
        "an exact duplicate tag must be 422, got {}: {}",
        r.status(),
        r.text().unwrap_or_default()
    );
    let body: serde_json::Value = serde_json::from_str(&r.text().unwrap()).unwrap_or_default();
    assert!(
        body.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .contains("Duplicate tag"),
        "the message must name the problem, got {body}"
    );

    // Case-insensitive duplicate: `tags.name` is citext, so "DUPALPHA" is the
    // same tag as "dupalpha".
    let r2 = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question with a cased duplicate tag?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["dupalpha", "DupAlpha"]}))
        .send()
        .unwrap();
    assert_eq!(
        r2.status().as_u16(),
        422,
        "a case-differing duplicate must be 422, got {}",
        r2.status()
    );

    // Whitespace differences are duplicates too.
    let r3 = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question with a padded duplicate tag?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["dupalpha", " dupalpha "]}))
        .send()
        .unwrap();
    assert_eq!(
        r3.status().as_u16(),
        422,
        "whitespace-padded duplicate must be 422, got {}",
        r3.status()
    );

    // Distinct tags must still be accepted.
    let ok = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Question with two distinct tags here?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["dupalpha", "dupbeta"]}))
        .send()
        .unwrap();
    assert_eq!(
        ok.status().as_u16(),
        200,
        "distinct tags must be accepted, got {}: {}",
        ok.status(),
        ok.text().unwrap_or_default()
    );
}

#[test]
fn answer_to_own_question_succeeds_without_notification() {
    // BEFORE: POST /answer built a notification email from `find_participants`
    // and called `.body(body).unwrap()`. When the author answers their own
    // question there are no *other* participants, so the Message ended up with
    // neither To nor Bcc, lettre returned `MissingTo`, and the worker panicked
    // - dropping the connection with no status line. Any logged-in user could
    // crash a request this way.
    //
    // NOW: an empty recipient set simply skips the mail and returns the
    // answer, because the answer itself is already committed at that point.
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let s = login_capture(&ctx, "selfans");
    ctx.ensure_tag("selfanstag");
    let q: serde_json::Value = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Answering my own question here?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["selfanstag"]}))
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
    assert!(!qid.is_empty(), "create-question qid, got {q}");
    let qid_i: i64 = qid.parse().expect("qid numeric");

    let r = s
        .auth(s.client.post(ctx.url("/answer")))
        .json(&json!({"id": qid,
            "value": "This answer value is definitely longer than twenty chars.",
            "reply_to": s.user_id.to_string()}))
        .send()
        .unwrap_or_else(|e| panic!("request failed outright instead of returning a status: {e}"));
    let status = r.status().as_u16();
    let text = r.text().unwrap_or_default();
    assert_eq!(
        status, 200,
        "a self-answer with no other participants must succeed, got {status}: {text}"
    );

    // The answer really is persisted - it was committed before the mail step.
    let body: serde_json::Value = serde_json::from_str(&text).expect("answer response JSON");
    let aid = body
        .get("data")
        .and_then(|d| d.as_str())
        .expect("answer id");
    let aid_i: i64 = aid.parse().expect("aid numeric");
    assert_eq!(
        ctx.q_i64("SELECT count(*) FROM posts WHERE id = $1", &[&aid_i]),
        1,
        "the self-answer must be stored"
    );
    // The question's answer counter is bumped exactly once.
    assert_eq!(
        ctx.q_i64("SELECT answer_count FROM posts WHERE id = $1", &[&qid_i]),
        1
    );

    // And the server is still healthy.
    let health = ctx.client.get(ctx.url("/get-questions-count/")).send();
    assert!(
        health.map(|r| r.status().is_success()).unwrap_or(false),
        "server must survive and keep serving"
    );
}

#[test]
fn cannot_vote_own_post() {
    let ctx = TestCtx::new();
    wait_for_backend_healthy(&ctx, 60);
    let s = login_capture(&ctx, "ownvote");
    ctx.ensure_tag("owntag");
    let q: serde_json::Value = s
        .auth(s.client.post(ctx.url("/create-question")))
        .json(&json!({
            "title": "Own vote question title?",
            "description": "This description is long enough to pass validation rules.",
            "tag_list": ["owntag"]}))
        .send()
        .unwrap()
        .json()
        .unwrap_or(serde_json::json!({}));
    let qid_i: i64 = q
        .get("data")
        .and_then(|d| d.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .parse()
        .expect("qid numeric");
    let v: serde_json::Value = s
        .auth(s.client.post(ctx.url("/votes")))
        .json(&json!({"vote": 1, "id": qid_i}))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert!(
        v.to_string().contains("own post"),
        "own vote rejected, got {v}"
    );
}

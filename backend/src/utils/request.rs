//! Helpers for turning untrusted request input into typed values without
//! panicking. A malformed value from the client must produce a 400, never a
//! worker panic (which aborts the connection without a status line).

use mobc_redis::redis::AsyncCommands;
use ntex::http::HttpMessage;
use ntex::web::{HttpRequest, HttpResponse};
use serde_json::json;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::state::AppStateRaw;
use crate::users::token;

pub const GENERIC_ERROR: &str = "Something went wrong, try again!";

pub fn bad_request(message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(&json!({"status": "fail", "message": message}))
}

/// A 500 that logs the real error server-side and tells the client nothing
/// about the database, schema or infrastructure behind it.
#[track_caller]
pub fn internal_error<E: std::fmt::Debug>(e: E) -> HttpResponse {
    let at = std::panic::Location::caller();
    error!("{}:{}: {:?}", at.file(), at.line(), e);
    HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": GENERIC_ERROR}))
}

/// Like [`internal_error`], for a failed DAO call. `status` is the value the
/// handler has always put in the body's `status` field (`"fail"` or `false`).
///
/// `RowNotFound` keeps its fixed sqlx message: the list pages in the frontend
/// match on that exact text to render "no results" rather than an alert. It
/// is a constant string and reveals nothing about the database.
#[track_caller]
pub fn db_error<E: std::fmt::Debug + 'static>(e: &E, status: serde_json::Value) -> HttpResponse {
    let any: &dyn std::any::Any = e;
    let message = match any.downcast_ref::<sqlx::Error>() {
        Some(row_not_found @ sqlx::Error::RowNotFound) => row_not_found.to_string(),
        _ => {
            let at = std::panic::Location::caller();
            error!("{}:{}: {:?}", at.file(), at.line(), e);
            GENERIC_ERROR.to_string()
        }
    };
    HttpResponse::InternalServerError().json(&json!({"status": status, "message": message}))
}

/// Parse a pagination cursor. An empty string means "start from now".
pub fn parse_cursor(s: &str) -> Result<OffsetDateTime, HttpResponse> {
    if s.is_empty() {
        return Ok(OffsetDateTime::now_utc());
    }
    OffsetDateTime::parse(s, &Rfc3339)
        .map_err(|_| bad_request("Invalid timestamp, expected RFC 3339."))
}

/// Parse a numeric id taken from the URL path.
pub fn parse_id(s: &str) -> Result<i64, HttpResponse> {
    s.trim().parse::<i64>().map_err(|_| bad_request("Invalid id."))
}

/// The id of the logged-in user for endpoints where login is optional
/// (e.g. reading a question), or 0 for anonymous requests.
///
/// The identity comes from the signed `access_token` JWT and the live session
/// in Redis - never from the `logged_in` cookie, which is plain unsigned JSON
/// that a client can rewrite to impersonate any user id.
pub async fn optional_user_id(req: &HttpRequest, state: &AppStateRaw) -> i64 {
    let Some(cookie) = req.cookie("access_token") else {
        return 0;
    };
    let details = match token::verify_jwt_token(
        state.config.access_token_public_key.to_owned(),
        cookie.value(),
    ) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    let Ok(mut redis) = state.kv.get().await else {
        return 0;
    };
    match redis.get::<_, String>(details.token_uuid.to_string()).await {
        Ok(uid) if uid.parse::<i64>().ok() == Some(details.user_id) => details.user_id,
        _ => 0,
    }
}

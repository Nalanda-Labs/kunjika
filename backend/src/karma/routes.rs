use super::dao;
use crate::state::AppState;
use crate::utils::request::{db_error, parse_keyset_cursor};

use ntex::web::{post, HttpResponse, Responder};
use serde_json::json;

#[derive(Deserialize)]
struct HistoryReq {
    per_page: Option<i64>,
    /// The `next_cursor` of the previous page; absent for the first page.
    cursor: Option<String>,
}

/// A user's karma history, newest first, cursor paginated. Public, like the
/// karma total on the profile; it says what each change was for and on which
/// post, never who voted.
#[post("/users/{id}/karma/")]
async fn history(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<HistoryReq>,
    state: AppState,
) -> impl Responder {
    let user = params.into_inner();
    let per_page = form.per_page.unwrap_or(20).clamp(1, 100);
    let cursor = match parse_keyset_cursor(form.cursor.as_deref()) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let karma = match dao::current(state.get_ref(), user).await {
        Ok(Some(k)) => k,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(&json!({"status": "fail", "message": "No such user."}))
        }
        Err(e) => return db_error(&e, json!("fail")),
    };
    match dao::history(state.get_ref(), user, per_page, cursor).await {
        Ok((events, next_cursor)) => HttpResponse::Ok()
            .json(&json!({"data": events, "next_cursor": next_cursor, "karma": karma})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

pub fn init(cfg: &mut ntex::web::ServiceConfig) {
    cfg.service(history);
}

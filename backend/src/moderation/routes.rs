use super::dao::{self, CloseOutcome, DeleteOutcome, SuspendOutcome};
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::utils::request::{db_error, parse_keyset_cursor};

use ntex::web::{get, post, HttpResponse, Responder};
use serde_json::json;

fn fail(message: &str) -> serde_json::Value {
    json!({"status": "fail", "message": message})
}

/// Delete a question or an answer.
///
/// Authors can delete their own answers, and their own questions as long as
/// nobody has answered them. Moderators can delete any post; a question
/// takes its answers with it. Deletion is permanent.
#[post("/delete-post/{id}")]
async fn delete_post(
    params: ntex::web::types::Path<i64>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    match dao::delete_post(state.get_ref(), params.into_inner(), &auth.user).await {
        Ok(DeleteOutcome::Deleted { question_id }) => {
            HttpResponse::Ok().json(&json!({"success": true, "question_id": question_id}))
        }
        Ok(DeleteOutcome::NotFound) => HttpResponse::NotFound().json(&fail("No such post.")),
        Ok(DeleteOutcome::Forbidden) => {
            HttpResponse::Forbidden().json(&fail("You can only delete your own post."))
        }
        Ok(DeleteOutcome::HasAnswers) => HttpResponse::Conflict().json(&fail(
            "This question has answers, so only a moderator can delete it. Report it if it should go.",
        )),
        Err(e) => db_error(&e, json!("fail")),
    }
}

/// The edit history of a post: its current text plus every earlier version,
/// newest first. Public, like the post itself.
#[get("/posts/{id}/history")]
async fn post_history(params: ntex::web::types::Path<i64>, state: AppState) -> impl Responder {
    match dao::post_history(state.get_ref(), params.into_inner()).await {
        Ok(Some(history)) => HttpResponse::Ok().json(&json!({"data": history})),
        Ok(None) => HttpResponse::NotFound().json(&fail("No such post.")),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[derive(Deserialize)]
struct CloseReq {
    /// Id of the question this one duplicates; omit for a plain close.
    duplicate_of: Option<i64>,
}

/// Close a question, optionally as a duplicate of another. Moderators only.
/// A closed question stays readable but accepts no new answers.
#[post("/questions/{id}/close/")]
async fn close_question(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<CloseReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    if !auth.is_moderator() {
        return HttpResponse::Forbidden().json(&fail("Only moderators can close questions."));
    }
    let qid = params.into_inner();
    match dao::close_question(state.get_ref(), qid, form.duplicate_of, auth.user.id).await {
        Ok(CloseOutcome::Done) => HttpResponse::Ok().json(&json!({"success": true})),
        Ok(CloseOutcome::NotFound) => HttpResponse::NotFound().json(&fail("No such question.")),
        Ok(CloseOutcome::DuplicateNotFound) => HttpResponse::UnprocessableEntity()
            .json(&fail("The original question was not found.")),
        Ok(CloseOutcome::SelfDuplicate) => HttpResponse::UnprocessableEntity()
            .json(&fail("A question cannot be a duplicate of itself.")),
        Err(e) => db_error(&e, json!("fail")),
    }
}

/// Reopen a closed question. Moderators only.
#[post("/questions/{id}/reopen/")]
async fn reopen_question(
    params: ntex::web::types::Path<i64>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    if !auth.is_moderator() {
        return HttpResponse::Forbidden().json(&fail("Only moderators can reopen questions."));
    }
    match dao::reopen_question(state.get_ref(), params.into_inner(), auth.user.id).await {
        Ok(CloseOutcome::NotFound) => HttpResponse::NotFound().json(&fail("No such question.")),
        Ok(_) => HttpResponse::Ok().json(&json!({"success": true})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[derive(Deserialize)]
struct SuspendReq {
    suspended: bool,
}

/// Suspend or reinstate a user. Moderators only; moderators cannot suspend
/// other moderators (a superuser can), and nobody can suspend a superuser.
#[post("/users/{id}/suspend/")]
async fn suspend_user(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<SuspendReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    if !auth.is_moderator() {
        return HttpResponse::Forbidden().json(&fail("Only moderators can suspend users."));
    }
    let target = params.into_inner();
    match dao::set_suspended(state.get_ref(), target, form.suspended, &auth.user).await {
        Ok(SuspendOutcome::Done) => {
            HttpResponse::Ok().json(&json!({"success": true, "suspended": form.suspended}))
        }
        Ok(SuspendOutcome::NotFound) => HttpResponse::NotFound().json(&fail("No such user.")),
        Ok(SuspendOutcome::SelfTarget) => {
            HttpResponse::UnprocessableEntity().json(&fail("You cannot suspend yourself."))
        }
        Ok(SuspendOutcome::Forbidden) => HttpResponse::Forbidden()
            .json(&fail("You are not allowed to suspend this user.")),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[derive(Deserialize)]
struct LogReq {
    per_page: Option<i64>,
    /// The `next_cursor` of the previous page; absent for the first page.
    cursor: Option<String>,
}

/// The moderation log: deletions, closes, reopens, suspensions and
/// reinstatements, newest first, cursor paginated. Moderators only.
#[post("/moderation/log/")]
async fn moderation_log(
    form: ntex::web::types::Json<LogReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    if !auth.is_moderator() {
        return HttpResponse::Forbidden()
            .json(&fail("Only moderators can view the moderation log."));
    }
    let per_page = form.per_page.unwrap_or(30).clamp(1, 100);
    let cursor = match parse_keyset_cursor(form.cursor.as_deref()) {
        Ok(c) => c,
        Err(r) => return r,
    };
    match dao::log_entries(state.get_ref(), per_page, cursor).await {
        Ok((entries, next_cursor)) => {
            HttpResponse::Ok().json(&json!({"data": entries, "next_cursor": next_cursor}))
        }
        Err(e) => db_error(&e, json!("fail")),
    }
}

pub fn init(cfg: &mut ntex::web::ServiceConfig) {
    cfg.service(moderation_log);
    cfg.service(delete_post);
    cfg.service(post_history);
    cfg.service(close_question);
    cfg.service(reopen_question);
    cfg.service(suspend_user);
}

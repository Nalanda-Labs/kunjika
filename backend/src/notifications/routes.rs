use super::dao;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::utils::request::{db_error, parse_keyset_cursor};

use ntex::web::{post, HttpResponse, Responder};
use serde_json::json;

#[derive(Deserialize)]
struct ListReq {
    per_page: Option<i64>,
    /// The `next_cursor` of the previous page; absent for the first page.
    cursor: Option<String>,
}

/// The signed-in user's notifications, newest first, cursor paginated.
#[post("/notifications/")]
async fn list(
    form: ntex::web::types::Json<ListReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let per_page = form.per_page.unwrap_or(20).clamp(1, 100);
    let cursor = match parse_keyset_cursor(form.cursor.as_deref()) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let page = dao::list(state.get_ref(), auth.user.id, per_page, cursor).await;
    let unread = dao::unread_count(state.get_ref(), auth.user.id).await;
    match (page, unread) {
        (Ok((items, next_cursor)), Ok(unread)) => HttpResponse::Ok()
            .json(&json!({"data": items, "next_cursor": next_cursor, "unread": unread})),
        (Err(e), _) | (_, Err(e)) => db_error(&e, json!("fail")),
    }
}

/// Number of unread notifications, for the bell in the navigation bar.
#[post("/notifications/unread-count/")]
async fn unread_count(auth: AuthorizationService, state: AppState) -> impl Responder {
    match dao::unread_count(state.get_ref(), auth.user.id).await {
        Ok(count) => HttpResponse::Ok().json(&json!({"count": count})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[derive(Deserialize)]
struct ReadReq {
    /// One notification; omit to mark everything read.
    id: Option<i64>,
}

#[post("/notifications/read/")]
async fn mark_read(
    form: ntex::web::types::Json<ReadReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    if let Err(e) = dao::mark_read(state.get_ref(), auth.user.id, form.id).await {
        return db_error(&e, json!("fail"));
    }
    match dao::unread_count(state.get_ref(), auth.user.id).await {
        Ok(unread) => HttpResponse::Ok().json(&json!({"success": true, "unread": unread})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[derive(Deserialize)]
struct SettingsReq {
    /// Omit to read the setting without changing it.
    email_notifications: Option<bool>,
}

/// Read or change whether "new answer" emails are sent to this user.
#[post("/notifications/settings/")]
async fn settings(
    form: ntex::web::types::Json<SettingsReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    match dao::email_setting(state.get_ref(), auth.user.id, form.email_notifications).await {
        Ok(enabled) => HttpResponse::Ok().json(&json!({"email_notifications": enabled})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[derive(Deserialize)]
struct FollowReq {
    following: bool,
}

/// Follow or unfollow a tag. New questions in a followed tag show up in the
/// follower's notifications.
#[post("/tags/follow/{tag}")]
async fn follow_tag(
    params: ntex::web::types::Path<String>,
    form: ntex::web::types::Json<FollowReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let tag = params.into_inner();
    match dao::set_following(state.get_ref(), auth.user.id, &tag, form.following).await {
        Ok(Some(following)) => HttpResponse::Ok().json(&json!({"following": following})),
        Ok(None) => HttpResponse::NotFound()
            .json(&json!({"status": "fail", "message": "No such tag."})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

/// The tags the signed-in user follows.
#[post("/tags/followed/")]
async fn followed_tags(auth: AuthorizationService, state: AppState) -> impl Responder {
    match dao::followed_tags(state.get_ref(), auth.user.id).await {
        Ok(tags) => HttpResponse::Ok().json(&json!({"data": tags})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

pub fn init(cfg: &mut ntex::web::ServiceConfig) {
    cfg.service(list);
    cfg.service(unread_count);
    cfg.service(mark_read);
    cfg.service(settings);
    cfg.service(follow_tag);
    cfg.service(followed_tags);
}

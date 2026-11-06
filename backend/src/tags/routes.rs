use super::dao::ITag;
use super::tag::*;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::utils::request::db_error;

use ntex::web::{self, get, post, HttpResponse, Responder};
use serde_json::json;
use validator::Validate;

#[post("/get-tags")]
async fn get_tags(
    form: web::types::Json<TagRequest>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let tag = form.into_inner();

    match state.get_ref().tag_query(&tag.tag).await {
        Ok(t) => {
            debug!("find tags {:?} ", t);

            HttpResponse::Ok().json(&json!({"data": t}))
        }
        Err(e) => {
            error!("find user error: {:?}", e);
            db_error(&e, json!("fail"))
        }
    }
}

#[post("/tags")]
async fn get_all_tags(form: web::types::Json<TagRequest>, state: AppState) -> impl Responder {
    let tag = form.into_inner();

    match state
        .get_ref()
        .get_all_tags_by_name(&tag.tag, &tag.direction)
        .await
    {
        Ok((t, count)) => {
            debug!("find tags {:?} ", t);

            HttpResponse::Ok().json(&json!({"data": t, "count": count}))
        }
        Err(e) => {
            debug!("tags error: {:?}", e);
            db_error(&e, json!("fail"))
        }
    }
}

#[get("/get-tag-info/{tag}")]
async fn get_tag_info(params: web::types::Path<String>, state: AppState) -> impl Responder {
    let tag = params.into_inner();

    match state.get_ref().get_tag_info(&tag).await {
        Ok(info) => HttpResponse::Ok().json(&json!({"info": info})),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound()
            .json(&json!({"status": "fail", "message": "No such tag."})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            db_error(&e, json!("fail"))
        }
    }
}

#[post("/tags/edit/{tag}")]
async fn update_tag_info(
    params: web::types::Path<String>,
    form: web::types::Json<TagInfoRequest>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    if !auth.is_moderator() {
        return HttpResponse::Forbidden().json(
            &json!({"status": "fail", "message": "Only moderators can edit tag info."}),
        );
    }

    let tag = params.into_inner();
    let tag_info = form.into_inner();

    // `TagInfoRequest` declares a minimum length that was never enforced.
    if let Err(e) = tag_info.validate() {
        return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": e.to_string()}));
    }
    match state
        .get_ref()
        .update_tag_info(&tag_info.tag_info, &tag)
        .await
    {
        Ok(_t) => HttpResponse::Ok().json(&json!({})),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound()
            .json(&json!({"status": "fail", "message": "No such tag."})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            db_error(&e, json!("fail"))
        }
    }
}

#[post("/search-tags")]
async fn search_tag(form: web::types::Json<SearchTagRequest>, state: AppState) -> impl Responder {
    let tag_req = form.into_inner();
    match state
        .get_ref()
        .search_tags(&tag_req.tag, tag_req.tags_per_page)
        .await
    {
        Ok(t) => HttpResponse::Ok().json(&json!({"data": t})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            db_error(&e, json!("fail"))
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tags);
    cfg.service(get_all_tags);
    cfg.service(get_tag_info);
    cfg.service(update_tag_info);
    cfg.service(search_tag);
}

use super::dao::ITag;
use super::tag::*;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

use ntex::web::{self, get, post, HttpResponse, Responder};
use serde_json::json;

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
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
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
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[get("/get-tag-info/{tag}")]
async fn get_tag_info(params: web::types::Path<String>, state: AppState) -> impl Responder {
    let tag = params.parse().unwrap();

    match state.get_ref().get_tag_info(&tag).await {
        Ok(info) => HttpResponse::Ok().json(&json!({"info": info})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[post("/tags/edit/{tag}")]
async fn update_tag_info(
    params: web::types::Path<String>,
    form: web::types::Json<TagInfoRequest>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let tag = params.parse().unwrap();
    let tag_info = form.into_inner();
    match state
        .get_ref()
        .update_tag_info(&tag_info.tag_info, &tag)
        .await
    {
        Ok(_t) => HttpResponse::Ok().json(&json!({})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
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
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
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

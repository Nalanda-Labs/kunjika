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
            HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[post("/tags")]
async fn get_all_tags(form: web::types::Json<TagRequest>, state: AppState) -> impl Responder {
    let tag = form.into_inner();

    match state.get_ref().get_all_tags(&tag.tag, tag.post_count).await {
        Ok(t) => {
            debug!("find tags {:?} ", t);

            HttpResponse::Ok().json(&json!({"data": t}))
        }
        Err(e) => {
            debug!("tags error: {:?}", e);
            HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[get("/tags/edit/{tag}/{id}")]
async fn get_tag_info(
    params: web::types::Path<(String, String)>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let id = params.1.parse().unwrap();

    match state.get_ref().get_tag_info(id).await {
        Ok(info) => HttpResponse::Ok().json(&json!({"data": info})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[post("/tags/edit/{tag}/{id}")]
async fn update_tag_info(
    params: web::types::Path<(String, String)>,
    form: web::types::Json<TagInfoRequest>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let id = params.1.parse().unwrap();
    let tag_info = form.into_inner();
    match state
        .get_ref()
        .update_tag_info(&tag_info.tag_info, id)
        .await
    {
        Ok(_t) => HttpResponse::Ok().json(&json!({"data": id})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tags);
    cfg.service(get_all_tags);
    cfg.service(get_tag_info);
    cfg.service(update_tag_info);
}

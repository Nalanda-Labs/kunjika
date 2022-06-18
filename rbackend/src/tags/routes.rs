use super::dao::ITag;
use super::tag::*;
use crate::api::ApiResult;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

use actix_web::{web, Responder, post, get};

#[post("/get-tags")]
async fn get_tags(
    form: web::Json<TagRequest>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let tag = form.into_inner();

    match state.get_ref().tag_query(&tag.tag).await {
        Ok(t) => {
            debug!("find tags {:?} ", t);

            return ApiResult::new().with_msg("ok").with_data(t);
        }
        Err(e) => {
            error!("find user error: {:?}", e);
            return ApiResult::new().code(500).with_msg(e.to_string());
        }
    }
}

#[post("/tags")]
async fn get_all_tags(
    form: web::Json<TagRequest>,
    state: AppState,
) -> impl Responder {
    let tag = form.into_inner();

    match state.get_ref().get_all_tags(&tag.tag, tag.post_count).await {
        Ok(t) => {
            debug!("find tags {:?} ", t);

            ApiResult::new().with_msg("ok").with_data(t)
        }
        Err(e) => {
            debug!("tags error: {:?}", e);
            ApiResult::new().code(400).with_msg(e.to_string())
        }
    }
}

#[get("/tags/edit/{tag}/{id}")]
async fn get_tag_info(
    params: web::Path<(String, String)>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let id = params.1.parse().unwrap();

    match state.get_ref().get_tag_info(id).await {
        Ok(info) => {
            ApiResult::new().with_msg("").with_data(info)
        }
        Err(e) => {
            debug!("update tag info: {:?}", e);
            ApiResult::new().code(500).with_msg(e.to_string())
        }
    }
}

#[post("/tags/edit/{tag}/{id}")]
async fn update_tag_info(
    params: web::Path<(String, String)>,
    form: web::Json<TagInfoRequest>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let id = params.1.parse().unwrap();
    let tag_info = form.into_inner();
    match state.get_ref().update_tag_info(&tag_info.tag_info, id).await {
        Ok(_t) => {
            ApiResult::new().with_msg("").with_data(id)
        }
        Err(e) => {
            debug!("update tag info: {:?}", e);
            ApiResult::new().code(500).with_msg(e.to_string())
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tags);
    cfg.service(get_all_tags);
    cfg.service(get_tag_info);
    cfg.service(update_tag_info);
}

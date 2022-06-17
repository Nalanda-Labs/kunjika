use super::dao::ITag;
use super::tag::TagRequest;
use crate::api::ApiResult;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

use actix_web::{web, Responder, post};

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

    match state.get_ref().get_all_tags(&tag.tag).await {
        Ok(t) => {
            debug!("find tags {:?} ", t);

            return ApiResult::new().with_msg("ok").with_data(t);
        }
        Err(e) => {
            debug!("tags error: {:?}", e);
            return ApiResult::new().code(400).with_msg(e.to_string());
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tags);
    cfg.service(get_all_tags);
}

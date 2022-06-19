use super::dao::IVote;
use super::vote::*;
use crate::api::ApiResult;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

use actix_web::{post, web, Responder};

#[post("/votes")]
async fn vote(
    form: web::Json<VoteRequest>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let data = form.into_inner();

    match state.get_ref().handle_vote(&data).await {
        Ok(r) => ApiResult::new().code(200).with_msg("").with_data(r),
        Err(e) => {
            debug!("{:?}", e.to_string());
            ApiResult::new().code(500).with_msg(e.to_string())
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(vote);
}

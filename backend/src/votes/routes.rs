use super::dao::IVote;
use super::vote::*;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

use ntex::web::{self, post, Responder, HttpResponse};
use serde_json::json;

#[post("/votes")]
async fn vote(
    form: web::types::Json<VoteRequest>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let data = form.into_inner();

    match state.get_ref().handle_vote(&data, &auth.user).await {
        Ok(r) => HttpResponse::Ok().json(&json!({"data": r})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            HttpResponse::InternalServerError().json(&json!({"data": e.to_string()}))
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(vote);
}

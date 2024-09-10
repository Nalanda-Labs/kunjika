use super::dao::IVote;
use super::vote::*;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

use ntex::web::{self, post, HttpResponse, Responder};
use serde_json::json;

#[post("/votes")]
async fn vote(
    form: web::types::Json<VoteRequest>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let data = form.into_inner();

    match state.get_ref().handle_vote(&data, &auth.user).await {
        Ok(r) => {
            if r.e == VoteEnum::VoteSuccessful {
                // do not send any data member for successful case
                HttpResponse::Ok().json(&json!({"success": true, "count_by_user": r.vote_by_user}))
            } else if r.e == VoteEnum::VoteOnce {
                HttpResponse::Ok()
                    .json(&json!({"success": false, "data": "You can vote only once."}))
            }
            // Usually this will not happen through browser but only through API calls
            else if r.e == VoteEnum::VoteYourOwnPost {
                HttpResponse::Ok()
                    .json(&json!({"success": false, "data": "You can't vote your own post."}))
            } else if r.e == VoteEnum::ReceivingUserNotFound {
                HttpResponse::Ok()
                    .json(&json!({"success": false, "data": "Receiviing user not found."}))
            } else {
                HttpResponse::InternalServerError().json(&json!({"data": "Unknown error!"}))
            }
        }
        Err(e) => {
            debug!("{:?}", e.to_string());
            HttpResponse::InternalServerError().json(&json!({"data": e.to_string()}))
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(vote);
}

use core::fmt;
use mobc_redis::redis::AsyncCommands;
use nonblock_logger::info;
use futures::future::Future;
use std::pin::Pin;
use actix_web::{dev, FromRequest, HttpRequest, Error};
use serde::{Deserialize, Serialize};

use crate::users::token;
use crate::users::user::User;
use crate::state::AppStateRaw;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct QueryParams {
    access_token: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[derive(Debug)]
pub struct AuthorizationService {
    pub user: User,
    pub xsrf_token: String,
}

impl FromRequest for AuthorizationService {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let state = req.app_data::<AppStateRaw>().unwrap().clone();
        let xsrf_token_header = req
            .headers()
            .get("X-XSRF-Token")
            .and_then(|h| h.to_str().ok());

        let xsrf_token = match xsrf_token_header {
            Some(x) => x.to_owned(),
            None => "".to_owned()
        };

        let mut access_token = match req.cookie("access_token") {
            Some(c) => c.to_string(),
            None => "".to_string()
        };

        Box::pin(async move{
            if xsrf_token == "" {
                return Err(actix_web::error::ErrorForbidden(format!(
                    "Wrong XSRF token!!!"
                )));
            }

            access_token = match access_token.split_once("=") {
                Some(v) => v.1.to_owned(),
                None => "".to_owned()
            };

            info!("Access token: {:?}", access_token);

            if access_token == "" {
                return Err(actix_web::error::ErrorForbidden(format!(
                    "Wrong access token!!!"
                )));
            }

            let access_token_details = match token::verify_jwt_token(
                state.config.access_token_public_key.to_owned(),
                &access_token,
            ) {
                Ok(token_details) => token_details,
                Err(e) => {
                    return Err(actix_web::error::ErrorForbidden(format!("{:?}", e.to_string())));
                }
            };

            info!("step 3");
            let access_token_uuid =
                uuid::Uuid::parse_str(&access_token_details.token_uuid.to_string()).unwrap();

            if xsrf_token != access_token_uuid.clone().to_string() {
                return Err(actix_web::error::ErrorForbidden(format!(
                    "Tokens do not match!!!"
                )));
            }

            info!("step 4");

            let redis_client = state.kv.get().await;
            let user_id;
            let uid;

            if redis_client.is_ok() {
                let mut rc = redis_client.unwrap();
                user_id = rc
                    .get::<_, String>(access_token_uuid.clone().to_string())
                    .await;

                if user_id.is_ok()  {
                    uid = user_id.unwrap();
                } else {
                    return Err(actix_web::error::ErrorForbidden(format!(
                        "User not found in db!!!"
                    )));
                }
            } else {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "DB Error!!!"
                )));
            }

            let query_result =
                sqlx::query_as!(User,
                                "SELECT id, username, email, password_hash, created_date, modified_date,
                            status, email_verified, image_url, designation, location, git,
                            website FROM users WHERE id = $1 and deleted=false",
                                uid.parse::<i64>().unwrap()
                )
                .fetch_optional(&state.sql).await;

            match query_result {
                Ok(Some(user)) => { return Ok(AuthorizationService { user, xsrf_token: access_token_uuid.to_string() });},
                Ok(None) => {
                    return Err(actix_web::error::ErrorInternalServerError(format!(
                        "The user belonging to this token no logger exists!"
                    )));
                },
                Err(e) => {
                    return Err(actix_web::error::ErrorInternalServerError(format!(
                        "Failed to check user existence: {:?}", e
                    )));
                }
            };
        })
    }
}

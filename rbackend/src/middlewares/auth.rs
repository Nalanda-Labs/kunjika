use actix_web::{dev, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::borrow::Cow;

use crate::api::ApiError;
use crate::state::AppStateRaw;
use crate::users::user::Claims;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct QueryParams {
    access_token: String,
}

#[derive(Debug)]
pub struct AuthorizationService {
    pub claims: Claims,
    pub xsrf_token: String,
}

impl FromRequest for AuthorizationService {
    type Error = ApiError;
    type Future = Ready<Result<AuthorizationService, Self::Error>>;

    // 1. header: Authorization: Bearer xxx
    // 2. URL's query: ?access_token=xxx
    // 3x. Body's query: ?access_token=xxx
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let xsrf_token_header = req
            .headers()
            .get("X-XSRF-Token")
            .and_then(|h| h.to_str().ok());
        let xsrf_token = match xsrf_token_header {
            Some(x) => x.to_owned(),
            None => "".to_owned(),
        };
        let token = req.cookie("jwt");

        match token
            .as_ref()
            .ok_or_else(|| Cow::Borrowed("Unauthorized"))
            .and_then(|token| {
                let state = req.app_data::<AppStateRaw>().expect("get AppStateRaw");
                let key = state.config.jwt_priv.as_bytes();
                match decode::<Claims>(
                    token.value(),
                    &DecodingKey::from_secret(key),
                    &Validation::new(Algorithm::HS256),
                ) {
                    Ok(claims) => {
                        if claims.claims.xsrf_token != xsrf_token {
                            Err(format!("Invalid credentials").into())
                        } else {
                            Ok(AuthorizationService {
                                claims: claims.claims,
                                xsrf_token: xsrf_token,
                            })
                        }
                    }
                    Err(e) => {
                        error!("jwt.decode {} failed: {:?}", token, e);
                        Err(format!("invalid token: {}", e).into())
                    }
                }
            }) {
            Ok(service) => ok(service),
            Err(e) => {
                let api = ApiError::new().code(400).with_msg(e);
                api.log(req);
                err(api)
            }
        }
    }
}

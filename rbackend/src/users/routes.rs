use super::dao::IUser;
use super::user::*;
use crate::api::ApiResult;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::utils::security::{check_signature, sign};

use actix_web::{cookie::Cookie, get, post, web, Error, HttpResponse, Responder};
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use uuid::Uuid;
use validator::Validate;

// curl -v --data '{"name": "Bob", "email": "Bob@google.com", "password": "Bobpass"}' -H "Content-Type: application/json" -X POST localhost:8080/user/register
#[post("/register")]
async fn register(form: web::Json<Register>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    if let Err(e) = form.validate() {
        error!("regitser {:?} error: {:?}", form, e);
        return ApiResult::new().code(400).with_msg(e.to_string());
    }
    if !form.match_password() {
        return ApiResult::new().code(400).with_msg("Passwords are bad!");
    }
    match state.get_ref().user_add(&form).await {
        Ok(res) => {
            info!("register {:?} res: {}", form, res);
            // TODO: move it out to a common logic
            let smtp_credentials = Credentials::new(
                state.config.mail_username.clone(),
                state.config.mail_password.clone(),
            );

            let mailer =
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&state.config.mail_host)
                    .unwrap()
                    .credentials(smtp_credentials)
                    .build();

            let from = state.config.from_name.clone() + "<" + &state.config.from_email + ">";
            let to = form.email.clone();
            let subject = "Registration at Kunjika";

            // Sign an arbitrary string.
            let token = sign(&form.email, &state).await;
            let body = format!(
                "Hi {},

Thank you for registering at Kunjika.
Your email confirmation link is https://{}/confirm-email/{}.
This email will expire in one day.

Thanks,
Shiv",
                form.username, state.config.host, token
            );
            debug!("{:?}, {:?}", from, to);
            let email = Message::builder()
                .from(from.parse().unwrap())
                .to(to.parse().unwrap())
                .subject(subject)
                .body(body.to_string())
                .unwrap();

            debug!("Sending email");
            match mailer.send(email).await {
                Ok(_r) => {
                    debug!("{:?}", _r);
                    ApiResult::new().with_msg("ok").with_data(res)
                }
                Err(_e) => {
                    debug!("{:?}", _e);
                    ApiResult::new().code(502).with_msg("ok").with_data(0)
                }
            }
        }
        Err(e) => {
            error!("regitser {:?} error: {:?}", form, e);
            ApiResult::new()
                .code(409)
                .with_msg("Either email or username is taken.")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub success: bool,
}

// curl -v --data '{"name": "Bob", "email": "Bob@google.com", "password": "Bobpass"}' -H "Content-Type: application/json" -X POST localhost:8080/user/login
#[post("/login")]
async fn login(form: web::Json<Login>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    use chrono::{DateTime, Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};

    // todo: distable login for deleted and blocked users
    match state.get_ref().user_query(&form.email).await {
        Ok(user) => {
            info!("find user {:?} ok: {:?}", form, user);

            if form.verify(&user.pass) {
                let exp: DateTime<Utc> = Utc::now()
                    + if form.rememberme {
                        Duration::days(30)
                    } else {
                        Duration::days(365 * 100)
                    };

                let uuid = Uuid::new_v4().to_string();
                let my_claims = Claims {
                    sub: user.username.clone(),
                    exp: exp.timestamp() as usize,
                    email: form.email,
                    username: user.username.clone(),
                    id: user.id,
                    xsrf_token: uuid,
                    image_url: user.image_url,
                };
                let key = state.config.jwt_priv.as_bytes();
                let token = encode(
                    &Header::default(),
                    &my_claims,
                    &EncodingKey::from_secret(key),
                )
                .unwrap();
                let r = LoginResponse { success: true };
                let resp = match serde_json::to_string(&r) {
                    Ok(json) => HttpResponse::Ok()
                        .cookie(
                            Cookie::build("jwt", token)
                                .domain("localhost")
                                .path("/")
                                .secure(true)
                                .http_only(true)
                                .finish(),
                        )
                        .content_type("application/json")
                        .body(json),
                    Err(e) => Error::from(e).into(),
                };
                resp
            } else {
                HttpResponse::Unauthorized().finish()
            }
        }
        Err(e) => {
            error!("find user {:?} error: {:?}", form, e);
            HttpResponse::BadRequest().finish()
        }
    }
}

#[post("/check-username-availability")]
async fn check_username_availability(form: web::Json<UserName>, state: AppState) -> impl Responder {
    match state.get_ref().user_query(&form.username).await {
        Ok(_user) => {
            debug!("User found, username unavailable");
            let res = AvailabilityResponse { success: false };
            ApiResult::new()
                .code(200)
                .with_msg("username unavailable")
                .with_data(res)
        }
        Err(e) => {
            debug!("{:?}", e.to_string());
            let res = AvailabilityResponse { success: true };
            ApiResult::new()
                .code(200)
                .with_msg("username available")
                .with_data(res)
        }
    }
}

#[get("/confirm-email/{token}")]
async fn confirm_email(form: web::Path<String>, state: AppState) -> impl Responder {
    let token = form.into_inner();
    let email = check_signature(&token, &state).await;
    if &email == "Signature was expired" {
        ApiResult::new()
            .code(400)
            .with_msg("Bad request")
            .with_data("".to_string())
    } else {
        match state.get_ref().user_query(&email).await {
            Ok(_user) => {
                debug!("User found, username unavailable");
                ApiResult::new().code(200).with_msg("Email verified")
            }
            Err(e) => {
                debug!("{:?}", e.to_string());
                ApiResult::new()
                    .code(400)
                    .with_msg("Your email is not registered with us!")
            }
        }
    }
}

#[post("/users")]
async fn get_users(form: web::Json<UsersReq>, state: AppState) -> impl Responder {
    let last_user = form.into_inner();
    match state.get_ref().get_users(&last_user).await {
        Ok(user_res) => {
            ApiResult::new().code(200).with_msg("").with_data(user_res)
        }
        Err(e) => {
            debug!("{:?}", e.to_string());
            ApiResult::new()
                .code(400)
                .with_msg("Bad request!")
        }
    }
}

#[get("/users/{id}/{username}")]
async fn get_profile(params: web::Path<(String, String)>, state: AppState) -> impl Responder {
    let uid = &params.0.parse::<i64>().unwrap();
    match state.get_ref().get_profile(&uid).await {
        Ok(profile) => {
            ApiResult::new().code(200).with_msg("").with_data(profile)
        }
        Err(e) => {
            debug!("{:?}", e.to_string());
            ApiResult::new()
                .code(400)
                .with_msg("Bad request!")
        }
    }
}


pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
    cfg.service(register);
    cfg.service(check_username_availability);
    cfg.service(confirm_email);
    cfg.service(get_users);
}

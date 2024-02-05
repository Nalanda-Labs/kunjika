use super::dao::IUser;
use super::user::*;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::utils::security::{check_signature, sign};
use crate::utils::verify_user::verify_profile_user;

use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use ntex::web::{self, get, post, HttpResponse, Responder, Error};
use cookie::Cookie;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

// curl -v --data '{"name": "Bob", "email": "Bob@google.com", "password": "Bobpass"}' -H "Content-Type: application/json" -X POST localhost:8080/user/register
#[post("/register")]
async fn register(form: web::types::Json<Register>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    if let Err(e) = form.validate() {
        error!("regitser {:?} error: {:?}", form, e);
        return HttpResponse::BadRequest().json(&json!({"status": "fail", "message": e.to_string()}));
    }
    if !form.match_password() {
        return HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": "Passwords are bad!"}));
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
                    HttpResponse::Ok().json(&json!({"data": res}))
                }
                Err(_e) => {
                    debug!("{:?}", _e);
                    HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": "Passwords are bad!"}))
                }
            }
        }
        Err(e) => {
            error!("regitser {:?} error: {:?}", form, e);
            HttpResponse::Conflict().json(&json!({"status": "fail", "message": "Either email or username is taken"}))
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub success: bool,
}

// curl -v --data '{"name": "Bob", "email": "Bob@google.com", "password": "Bobpass"}' -H "Content-Type: application/json" -X POST localhost:8080/user/login
#[post("/login")]
async fn login(form: web::types::Json<Login>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    use chrono::{DateTime, Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};

    // todo: distable login for deleted and blocked users
    match state.get_ref().user_query(&form.email).await {
        Ok(user) => {
            info!("find user {:?} ok: {:?}", form, user);

            if user.email_verified == Some(false) {
                return HttpResponse::Unauthorized().finish();
            }

            if form.verify(&user.password_hash) {
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
                            Cookie::build("jwt", token.clone())
                                .domain("localhost")
                                .path("/")
                                .secure(true)
                                .http_only(true)
                                .finish().to_string()
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
            debug!("find user {:?} error: {:?}", form, e);
            HttpResponse::Unauthorized().finish()
        }
    }
}

#[post("/check-username-availability")]
async fn check_username_availability(form: web::types::Json<UserName>, state: AppState) -> impl Responder {
    match state.get_ref().user_query(&form.username).await {
        Ok(_user) => {
            debug!("User found, username unavailable");
            let res = AvailabilityResponse { success: false };
            HttpResponse::Ok().json(&json!({"data": res, "message": "username available"}))
        }
        Err(e) => {
            debug!("{:?}", e.to_string());
            HttpResponse::Ok().json(&json!({"status": "fail", "message": "username unavailable"}))
        }
    }
}

#[get("/confirm-email/{token}")]
async fn confirm_email(form: web::types::Path<String>, state: AppState) -> impl Responder {
    let token = form.into_inner();
    let email = check_signature(&token, &state).await;
    if &email == "Signature was expired" {
        HttpResponse::BadRequest().json(&json!({"status": "fail", "message": "Signature has expired!"}))
    } else {
        match state.get_ref().verify_email(&email).await {
            Ok(_user) => {
                debug!("User found, username unavailable");
                HttpResponse::Ok().json(&json!({"message": "username available"}))
            }
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::BadRequest().json(&json!({"status": "fail", "message": "Your email is not registered with us!"}))
            }
        }
    }
}

#[post("/users")]
async fn get_users(form: web::types::Json<UsersReq>, state: AppState) -> impl Responder {
    let last_user = form.into_inner();
    match state.get_ref().get_users(&last_user).await {
        Ok(user_res) => HttpResponse::Ok().json(&json!({"message": "", "data": user_res})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[get("/user/{id}/{username}")]
async fn get_profile(params: web::types::Path<(String, String)>, state: AppState) -> impl Responder {
    let uid = &params.0.parse::<i64>().unwrap();
    match state.get_ref().get_profile(&uid).await {
        Ok(profile) => HttpResponse::Ok().json(&json!({"message": "", "data": profile})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[get("/profile/{id}/username/{username}")]
async fn update_username(
    params: web::types::Path<(String, String)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = params.0.parse::<i64>().unwrap();
    let username = params.1.parse().unwrap();
    let user = verify_profile_user(uid, &auth).await;
    if user {
        match state.get_ref().update_username(uid, &username).await {
            Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
            }
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[get("/profile/{id}/title/{title}")]
async fn update_title(
    params: web::types::Path<(String, String)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = params.0.parse::<i64>().unwrap();
    let title = params.1.parse().unwrap();
    let user = verify_profile_user(uid, &auth).await;
    if user {
        match state.get_ref().update_title(uid, &title).await {
            Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
            }
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[get("/profile/{id}/name/{name}")]
async fn update_name(
    params: web::types::Path<(String, String)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = params.0.parse::<i64>().unwrap();
    let name = params.1.parse().unwrap();
    let user = verify_profile_user(uid, &auth).await;
    if user {
        match state.get_ref().update_name(uid, &name).await {
            Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
            }
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[get("/profile/{id}/designation/{designation}")]
async fn update_designation(
    params: web::types::Path<(String, String)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = params.0.parse::<i64>().unwrap();
    let designation = params.1.parse().unwrap();
    let user = verify_profile_user(uid, &auth).await;
    if user {
        match state.get_ref().update_designation(uid, &designation).await {
            Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
            }
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[get("/profile/{id}/location/{location}")]
async fn update_location(
    params: web::types::Path<(String, String)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = params.0.parse::<i64>().unwrap();
    let location = params.1.parse().unwrap();
    let user = verify_profile_user(uid, &auth).await;
    if user {
        match state.get_ref().update_location(uid, &location).await {
            Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
            }
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[get("/edit-links/{uid}")]
async fn get_links(
    params: web::types::Path<String>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = params.parse::<i64>().unwrap();
    let user = verify_profile_user(uid, &auth).await;
    if user {
        match state.get_ref().get_links(uid).await {
            Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
            }
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[post("/edit-links/{uid}")]
async fn update_links(
    params: web::types::Path<String>,
    form: web::types::Json<LinksResponse>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = params.parse::<i64>().unwrap();
    let data = form.into_inner();
    let user = verify_profile_user(uid, &auth).await;
    if user {
        match state.get_ref().update_links(uid, &data).await {
            Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::InternalServerError().json(&json!({"status": "fail", "message": e.to_string()}))
            }
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
    cfg.service(register);
    cfg.service(check_username_availability);
    cfg.service(confirm_email);
    cfg.service(get_users);
    cfg.service(get_profile);
    cfg.service(update_username);
    cfg.service(update_title);
    cfg.service(update_name);
    cfg.service(update_designation);
    cfg.service(update_location);
    cfg.service(get_links);
    cfg.service(update_links);
}

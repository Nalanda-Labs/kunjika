use super::dao::IUser;
use super::user::{Claims, Login, Register};
use crate::api::ApiResult;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::utils::security::sign;

use actix_web::{cookie::Cookie, delete as del, get, post, web, Error, HttpResponse, Responder};
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
                    sub: user.name.clone(),
                    exp: exp.timestamp() as usize,
                    email: form.email,
                    username: user.name.clone(),
                    id: user.id,
                    xsrf_token: uuid,
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

// curl -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJCb2IiLCJleHAiOjE1OTEyNDYwOTR9.O1dbYu3tqiIi6I8OUlixLuj9dp-1tLl4mjmXZ0ve6uo' localhost:8080/user/info/who |jq .
// curl 'localhost:8080/user/userInfo?access_token=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJCb2IiLCJleHAiOjE1OTEyNTYxNDd9.zJKlZOozYfq-xMXO89kjUyme6SA8_eziacqt5gvXj2U' |jq .
#[get("/info/{who}")]
async fn info(
    form: web::Path<String>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let who = form.into_inner();
    let w = who.as_str();

    // me
    let user = match state.get_ref().user_query(&auth.claims.sub).await {
        Ok(user) => {
            debug!("find user {:?} ok: {:?}", auth.claims, user);

            if who == "_"
                || [
                    user.id.to_string().as_str(),
                    user.name.as_str(),
                    user.email.as_str(),
                ]
                .contains(&w)
            {
                return ApiResult::new().with_msg("ok").with_data(user);
            }

            user
        }
        Err(e) => {
            error!("find user {:?} error: {:?}", auth.claims, e);
            return ApiResult::new().code(500).with_msg(e.to_string());
        }
    };

    // todo: add role(admin, user, guest)
    if user.status != "normal" {
        return ApiResult::new().code(403);
    }

    match state.get_ref().user_query(w).await {
        Ok(user) => {
            debug!("find user {:?} ok: {:?}", w, user);
            ApiResult::new().with_msg("ok").with_data(user)
        }
        Err(e) => {
            error!("find user {:?} error: {:?}", w, e);
            ApiResult::new().code(500).with_msg(e.to_string())
        }
    }
}

// curl -v -X DELETE localhost:8080/user/who
#[del("/delete/{who}")]
async fn delete(
    form: web::Path<String>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let user = match state.get_ref().user_query(&auth.claims.sub).await {
        Ok(user) => user,
        Err(e) => {
            error!("find user {:?} error: {:?}", auth.claims, e);
            return ApiResult::new().code(500).with_msg(e.to_string());
        }
    };

    // todo: add role(admin, user, guest)
    if user.status != "normal" {
        return ApiResult::new().code(403);
    }

    let who = form.into_inner();
    match state.get_ref().user_delete(&who).await {
        Ok(res) => {
            info!(
                "delete {:?} res: {} {} {} {}",
                who, res.id, res.name, res.email, res.status
            );
            ApiResult::new().with_msg("ok").with_data(res)
        }
        Err(e) => {
            error!("delete {:?} error: {:?}", who, e);
            ApiResult::new().code(400).with_msg(e.to_string())
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
    cfg.service(register);
    cfg.service(delete);
    cfg.service(info);
}

use super::dao::IUser;
use super::user::*;
use crate::middlewares::auth::{self, AuthorizationService};
use crate::state::AppState;
use crate::users::token;
use crate::uploads::save::save_upload;
use crate::utils::request::{db_error, internal_error, parse_id};
use crate::utils::security::{check_signature, extract_email, sign, sign_in_code, SignatureError};
use crate::utils::verify_user::verify_profile_user;

use cookie::time::Duration;
use cookie::Cookie;
use lettre::{AsyncTransport, Message};
use mobc_redis::redis::{self, AsyncCommands};
use ntex::http::HttpMessage;
use ntex::web::{self, get, post, Error, HttpRequest, HttpResponse, Responder};
use ntex_multipart::Multipart;
use serde_json::json;
use validator::Validate;

/// The `Domain` attribute to stamp on a session cookie.
///
/// RFC 6265 forbids a port in `Domain`, and browsers reject the entire cookie
/// when one is present. `Config::host` is `localhost:5173` in the shipped
/// `template.json`, so the port has to be stripped here - the port belongs in
/// the URL, not in the cookie. Without this the session cookie is silently
/// dropped and the user never appears logged in.
fn cookie_domain(host: &str) -> String {
    let trimmed = host.trim();
    let bare = trimmed.split(':').next().unwrap_or(trimmed).trim();
    // A misconfigured/empty `host` must not crash the handler; fall back to
    // whatever was configured rather than inventing a value.
    if bare.is_empty() {
        trimmed.to_string()
    } else {
        bare.to_string()
    }
}

#[post("/register")]
async fn register(form: web::types::Json<Register>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    if let Err(e) = form.validate() {
        error!("regitser {:?} error: {:?}", form, e);
        return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": e.to_string()}));
    }

    if !form.check_email() {
        return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": "+ in email not allowed!"}));
    }

    match state.get_ref().user_add(&form).await {
        Ok(res) => {
            debug!("register {:?} res: {}", form, res);

            let from = state.config.from_name.clone() + "<" + &state.config.from_email + ">";
            let to = form.email.clone();
            let subject = "Registration at Kunjika";

            // Sign an arbitrary string.
            let token = sign(&form.email, &state).await;
            match state
                .get_ref()
                .save_confirmation_token(&form.email, &token)
                .await
            {
                Ok(_t) => {
                    let body = format!(
                        "Hi {},

Thank you for registering at Kunjika.
Your email confirmation link is https://{}/confirm-email/{}.
This link will expire in 5 min.

Thanks,
Team Kunjika",
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
                    match &state.mailer.send(email).await {
                        Ok(_r) => {
                            debug!("{:?}", _r);
                            HttpResponse::Ok().json(&json!({"data": res}))
                        }
                        Err(_e) => {
                            info!("{:?}", _e);
                            HttpResponse::InternalServerError().json(
                            &json!({"status": "fail", "message": "Something went wrong, try again!"}),
                        )
                        }
                    }
                }
                Err(_e) => {
                    info!("{:?}", _e);
                    HttpResponse::InternalServerError().json(
                        &json!({"status": "fail", "message": "Something went wrong, try again!"}),
                    )
                }
            }
        }
        Err(e) => {
            // Only a genuine unique violation means "already taken". Mapping
            // *every* DAO error here used to report SQLSTATE 22001 (value too
            // long) as 409 "Either email or username is taken", which hid real
            // database problems behind a misleading conflict message.
            if is_unique_violation(&e) {
                HttpResponse::Conflict().json(
                    &json!({"status": "fail", "message": "Either email or username is taken"}),
                )
            } else {
                error!("register {:?} database error: {:?}", form, e);
                HttpResponse::InternalServerError()
                    .json(&json!({"status": "fail", "message": "Something went wrong, try again!"}))
            }
        }
    }
}

/// SQLSTATE 23505 - unique_violation. Raised by the `users_name_key` UNIQUE
/// index on `username` and by the unique index on `users.email`.
const UNIQUE_VIOLATION: &str = "23505";

/// True only when `err` is a PostgreSQL/YugabyteDB unique violation.
fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db| db.code())
        .map(|code| code == UNIQUE_VIOLATION)
        .unwrap_or(false)
}

#[post("/submit-email")]
async fn submit_email_for_login(form: web::types::Json<Login>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    match state.get_ref().get_email_for_login(&form.email).await {
        Ok(true) => {
            let from = state.config.from_name.clone() + "<" + &state.config.from_email + ">";
            let to = form.email.clone();
            let subject = "Sign in code for Kunjika";

            let login_otp = sign_in_code(&state.config.login_code_length);
            match state
                .get_ref()
                .save_confirmation_token(&form.email, &login_otp)
                .await
            {
                Ok(_t) => {
                    let body = format!(
                        "Hi,

The OTP for your login is {}.
Notte that this OTP is valid for 5 min.
If this is not generated by you please report it to
https://{}

Thanks,
Team Kunjika",
                        login_otp, state.config.host
                    );
                    debug!("{:?}, {:?}", from, to);
                    let email = Message::builder()
                        .from(from.parse().unwrap())
                        .to(to.parse().unwrap())
                        .subject(subject)
                        .body(body.to_string())
                        .unwrap();

                    debug!("Sending email");
                    match &state.mailer.send(email).await {
                        Ok(_r) => {
                            debug!("{:?}", _r);
                            HttpResponse::Ok().json(&json!({"success": true}))
                        }
                        Err(_e) => {
                            info!("{:?}", _e);
                            HttpResponse::InternalServerError().json(
                            &json!({"status": "fail", "message": "Something went wrong, try again!"}),
                        )
                        }
                    }
                }
                Err(_e) => {
                    info!("{:?}", _e);
                    HttpResponse::InternalServerError().json(
                        &json!({"status": "fail", "message": "Something went wrong, try again!"}),
                    )
                }
            }
        }
        Ok(false) => {
            debug!("find user {:?}", form);
            HttpResponse::Unauthorized().finish()
        }
        Err(e) => {
            debug!("find user {:?} error: {:?}", form, e);
            HttpResponse::Unauthorized().finish()
        }
    }
}

#[post("/auth/login")]
async fn login(form: web::types::Json<LoginForm>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    // todo: distable login for deleted and blocked users
    match state.get_ref().user_query(&form.email).await {
        Ok(user) => {
            info!("login attempt for user {}", user.id);
            match state
                .get_ref()
                .validate_login_otp(&form.email, &form.otp)
                .await
            {
                Ok(true) => {
                    let access_token_details = match token::generate_jwt_token(
                        user.id,
                        state.config.access_token_max_age,
                        state.config.access_token_private_key.to_owned(),
                    ) {
                        Ok(token_details) => token_details,
                        Err(e) => return internal_error(e),
                    };

                    info!("Access token built");
                    let refresh_token_details = match token::generate_jwt_token(
                        user.id,
                        state.config.refresh_token_max_age,
                        state.config.refresh_token_private_key.to_owned(),
                    ) {
                        Ok(token_details) => token_details,
                        Err(e) => return internal_error(e),
                    };

                    let mut redis_client = match state.kv.get().await {
                        Ok(redis_client) => redis_client,
                        Err(e) => return internal_error(e),
                    };

                    let access_result: redis::RedisResult<()> = redis_client
                        .set_ex(
                            access_token_details.token_uuid.to_string(),
                            user.id.to_string(),
                            (state.config.access_token_max_age * 60) as u64,
                        )
                        .await;

                    if let Err(e) = access_result {
                        return internal_error(e);
                    }

                    let refresh_result: redis::RedisResult<()> = redis_client
                        .set_ex(
                            refresh_token_details.token_uuid.to_string(),
                            user.id.to_string(),
                            (state.config.refresh_token_max_age * 60) as u64,
                        )
                        .await;

                    if let Err(e) = refresh_result {
                        return internal_error(e);
                    }

                    drop(redis_client);

                    let access_cookie = Cookie::build((
                        "access_token",
                        access_token_details.token.clone().unwrap(),
                    ))
                    .domain(cookie_domain(&state.config.host))
                    .path("/")
                    .secure(true)
                    .max_age(Duration::seconds(state.config.access_token_max_age * 60))
                    .http_only(true)
                    .same_site(cookie::SameSite::None);
                    let refresh_cookie =
                        Cookie::build(("refresh_token", refresh_token_details.token.unwrap()))
                            .domain(cookie_domain(&state.config.host))
                            .path("/")
                            .secure(true)
                            .max_age(Duration::seconds(state.config.refresh_token_max_age * 60))
                            .http_only(true)
                            .same_site(cookie::SameSite::None);
                    let xsrf_cookie =
                        Cookie::build(("xsrf_token", access_token_details.token_uuid.to_string()))
                            .domain(cookie_domain(&state.config.host))
                            .max_age(Duration::seconds(state.config.access_token_max_age * 60))
                            .path("/")
                            .secure(true)
                            .http_only(false)
                            .same_site(cookie::SameSite::None);

                    let r: LoginResponse = LoginResponse {
                        user,
                        success: true,
                    };
                    let resp = match serde_json::to_string(&r) {
                        Ok(json) => HttpResponse::Ok()
                            .cookie(access_cookie)
                            .cookie(refresh_cookie)
                            .cookie(xsrf_cookie)
                            .cookie(
                                Cookie::build(("logged_in", json))
                                    .domain(cookie_domain(&state.config.host))
                                    .path("/")
                                    .secure(true)
                                    .http_only(true)
                                    .max_age(Duration::seconds(
                                        state.config.access_token_max_age * 60,
                                    ))
                                    .same_site(cookie::SameSite::None),
                            )
                            .content_type("application/json")
                            .body(""),
                        Err(e) => Error::from(e).into(),
                    };
                    resp
                }
                Ok(false) => {
                    info!("login: wrong or expired code for {}", form.email);
                    HttpResponse::Unauthorized().finish()
                }
                Err(e) => {
                    error!("login: code check failed for {}: {:?}", form.email, e);
                    HttpResponse::Unauthorized().finish()
                }
            }
        }
        Err(e) => {
            info!("login: no user {}: {:?}", form.email, e);
            HttpResponse::Unauthorized().finish()
        }
    }
}

#[get("/auth/refresh")]
async fn refresh_access_token_handler(req: HttpRequest, state: AppState) -> impl Responder {
    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => {
            info!("step 1");
            return delete_cookie();
        }
    };

    let refresh_token_details = match token::verify_jwt_token(
        state.config.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(_e) => {
            info!("step 2");
            return delete_cookie();
        }
    };

    let result = state.kv.get().await;
    let mut redis_client = match result {
        Ok(redis_client) => redis_client,
        Err(_e) => {
            info!("step 3");
            return delete_cookie();
        }
    };
    let redis_result: redis::RedisResult<String> = redis_client
        .get(refresh_token_details.token_uuid.to_string())
        .await;

    let user_id = match redis_result {
        Ok(value) => value,
        Err(_e) => {
            info!("step 4");
            return delete_cookie();
        }
    };

    let user_id = match user_id.parse::<i64>() {
        Ok(id) => id,
        Err(_e) => return delete_cookie(),
    };
    let query_result = sqlx::query_as!(
        User,
        "SELECT id, username, email, created_date, image_url, email_verified,
        modified_date, status, designation, git, location, website, is_superuser, is_moderator FROM users
        WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.sql)
    .await;

    let user = match query_result {
        Ok(Some(user)) => user,
        Ok(None) => return delete_cookie(),
        Err(e) => {
            error!("refresh: user lookup failed: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": "Something went wrong, try again!"}));
        }
    };

    let access_token_details = match token::generate_jwt_token(
        user.id,
        state.config.access_token_max_age,
        state.config.access_token_private_key.to_owned(),
    ) {
        Ok(token_details) => token_details,
        Err(_e) => {
            return delete_cookie();
        }
    };

    let new_refresh_token_details = match token::generate_jwt_token(
        user.id,
        state.config.refresh_token_max_age,
        state.config.refresh_token_private_key.to_owned(),
    ) {
        Ok(token_details) => token_details,
        Err(_e) => {
            return delete_cookie();
        }
    };

    let redis_result: redis::RedisResult<()> = redis_client
        .set_ex(
            new_refresh_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (state.config.refresh_token_max_age * 60) as u64,
        )
        .await;

    if redis_result.is_err() {
        return delete_cookie();
    }

    let redis_result: redis::RedisResult<()> = redis_client
        .set_ex(
            access_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (state.config.access_token_max_age * 60) as u64,
        )
        .await;

    if redis_result.is_err() {
        return delete_cookie();
    }

    // Rotate: retire the refresh token that was just spent, and the access
    // token it replaces, so a stolen copy of either stops working shortly
    // instead of living out its full TTL alongside the new pair. They expire
    // after a short grace period rather than at once: a page load refreshes
    // the session while other requests (or other tabs) may still be in
    // flight with the old cookies, and those must not fail with 401.
    let mut retired = vec![refresh_token_details.token_uuid.to_string()];
    if let Some(c) = req.cookie("access_token") {
        if let Ok(old) =
            token::verify_jwt_token(state.config.access_token_public_key.to_owned(), c.value())
        {
            retired.push(old.token_uuid.to_string());
        }
    }
    for key in &retired {
        let r: redis::RedisResult<bool> = redis_client.expire(key, ROTATION_GRACE_SECS).await;
        if let Err(e) = r {
            error!("refresh: could not retire old token: {:?}", e);
        }
    }

    drop(redis_client);

    let access_cookie =
        Cookie::build(("access_token", access_token_details.token.clone().unwrap()))
            .domain(cookie_domain(&state.config.host))
            .path("/")
            .secure(true)
            .max_age(Duration::seconds(state.config.access_token_max_age * 60))
            .http_only(true)
            .same_site(cookie::SameSite::None);

    let refresh_cookie = Cookie::build(("refresh_token", new_refresh_token_details.token.unwrap()))
        .domain(cookie_domain(&state.config.host))
        .path("/")
        .secure(true)
        .max_age(Duration::seconds(state.config.refresh_token_max_age * 60))
        .http_only(true)
        .same_site(cookie::SameSite::None);

    let xsrf_cookie = Cookie::build(("xsrf_token", access_token_details.token_uuid.to_string()))
        .domain(cookie_domain(&state.config.host))
        .path("/")
        .secure(true)
        .max_age(cookie::time::Duration::seconds(
            state.config.access_token_max_age * 60,
        ))
        .http_only(false)
        .same_site(cookie::SameSite::None);

    let r: LoginResponse = LoginResponse {
        user,
        success: true,
    };
    let resp = match serde_json::to_string(&r) {
        Ok(json) => HttpResponse::Ok()
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .header("Pragma", "no-cache")
            .header("Expires", 0)
            .cookie(access_cookie)
            .cookie(refresh_cookie)
            .cookie(xsrf_cookie)
            .cookie(
                Cookie::build(("logged_in", json))
                    .domain(cookie_domain(&state.config.host))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .max_age(Duration::seconds(state.config.access_token_max_age * 60))
                    .same_site(cookie::SameSite::None),
            )
            .content_type("application/json")
            .body(""),
        Err(e) => Error::from(e).into(),
    };
    resp
}

/// How long a rotated-out token keeps working after a refresh (seconds).
const ROTATION_GRACE_SECS: i64 = 10;

fn delete_cookie() -> HttpResponse {
    // in case of an error we simply delete cookie because it is not possible
    // to recover from the error
    let access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .max_age(Duration::new(-1, 0))
        .http_only(true);
    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .max_age(Duration::new(-1, 0))
        .http_only(true);
    let logged_in_cookie = Cookie::build(("logged_in", ""))
        .path("/")
        .max_age(Duration::new(-1, 0))
        .http_only(true);

    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .cookie(logged_in_cookie)
        .json(&json!({"status": true}))
}

#[post("/auth/logout")]
async fn logout_handler(
    req: HttpRequest,
    auth_guard: auth::AuthorizationService,
    state: AppState,
) -> impl Responder {
    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => return delete_cookie(),
    };

    let access_token = match req.cookie("access_token") {
        Some(c) => c.value().to_string(),
        None => return delete_cookie(),
    };

    let refresh_token_details = match token::verify_jwt_token(
        state.config.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(_e) => return delete_cookie(),
    };

    let access_token_details = match token::verify_jwt_token(
        state.config.access_token_public_key.to_owned(),
        &access_token,
    ) {
        Ok(token_details) => token_details,
        Err(_e) => return delete_cookie(),
    };

    let mut redis_client = match state.kv.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("logout: redis unavailable: {:?}", e);
            return delete_cookie();
        }
    };
    let _redis_result: redis::RedisResult<usize> = redis_client
        .del(&[
            access_token_details.token_uuid.to_string(),
            refresh_token_details.token_uuid.to_string(),
            auth_guard.xsrf_token,
        ])
        .await;

    delete_cookie()
}

#[post("/check-username-availability")]
async fn check_username_availability(
    form: web::types::Json<UserName>,
    state: AppState,
) -> impl Responder {
    match state.get_ref().user_query(&form.username).await {
        Ok(_user) => {
            debug!("User found, username unavailable");
            let res = AvailabilityResponse { success: false };
            HttpResponse::Ok().json(&json!({"data": res, "message": "username unavailable"}))
        }
        Err(e) => {
            debug!("{:?}", e.to_string());
            HttpResponse::Ok().json(&json!({"status": true, "message": "username available"}))
        }
    }
}

#[get("/confirm-email/{token}")]
async fn confirm_email(form: web::types::Path<String>, state: AppState) -> impl Responder {
    let token = form.into_inner();
    let email = match check_signature(&token, &state).await {
        Ok(email) => email,
        Err(SignatureError::Expired) => {
            return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": "Signature has expired!
            You can get another token by clicking the resend email button."}))
        }
        Err(SignatureError::Invalid) => {
            return HttpResponse::BadRequest()
                .json(&json!({"status": "fail", "message": "Invalid signature!"}))
        }
    };
    {
        match state.get_ref().verify_email(&email, &token).await {
            Ok(t) => {
                if t {
                    debug!("Email verified!");
                    HttpResponse::Ok().json(&json!({"success": true, "message": "Email verified."}))
                } else {
                    debug!("Token not found!");
                    HttpResponse::Ok()
                        .json(&json!({"success": false, "message": "Invalid or expired token!"}))
                }
            }
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::BadRequest()
                    .json(&json!({"status": false, "message": "Email not found!"}))
            }
        }
    }
}

#[get("/resend-confirmation-email/{token}")]
async fn resend_confirmation_email(
    form: web::types::Path<String>,
    state: AppState,
) -> impl Responder {
    let token = form.into_inner();
    let email = match extract_email(&token, &state).await {
        Ok(email) => email,
        Err(_) => {
            return HttpResponse::BadRequest().json(
                &json!({"status": "fail", "message": "Bad signature. Use link from your email."}),
            )
        }
    };
    {
        let from = state.config.from_name.clone() + "<" + &state.config.from_email + ">";
        let to = email.clone();
        let subject = "Registration at Kunjika";

        // Sign an arbitrary string.
        let token = sign(&email, &state).await;
        match state
            .get_ref()
            .save_confirmation_token(&email, &token)
            .await
        {
            Ok(_t) => {
                // TODO: Get username from db to add to the email
                let body = format!(
                    "Hi,

Thank you for registering at Kunjika.
Your email confirmation link is https://{}/confirm-email/{}.
This link will expire in one day.

Thanks,
Team Kunjika",
                    state.config.host, token
                );
                debug!("{:?}, {:?}", from, to);
                let email = Message::builder()
                    .from(from.parse().unwrap())
                    .to(to.parse().unwrap())
                    .subject(subject)
                    .body(body.to_string())
                    .unwrap();

                debug!("Sending email");
                match &state.mailer.send(email).await {
                    Ok(_r) => {
                        debug!("{:?}", _r);
                        HttpResponse::Ok()
                            .json(&json!({"success": true, "message": "Confirmation email sent!"}))
                    }
                    Err(e) => internal_error(e),
                }
            }
            Err(_e) => {
                info!("{:?}", _e);
                db_error(&_e, json!(false))
            }
        }
    }
}

#[post("/users")]
async fn get_users(form: web::types::Json<UsersReq>, state: AppState) -> impl Responder {
    let last_user = form.into_inner();
    match state.get_ref().get_users(&last_user).await {
        Ok((user_res, count)) => {
            HttpResponse::Ok().json(&json!({"message": "", "data": user_res, "count": count}))
        }
        Err(e) => {
            debug!("{:?}", e.to_string());
            db_error(&e, json!("fail"))
        }
    }
}

#[get("/user/{id}/{username}")]
async fn get_profile(
    params: web::types::Path<(String, String)>,
    state: AppState,
) -> impl Responder {
    let uid = &match parse_id(&params.0) {
        Ok(id) => id,
        Err(r) => return r,
    };
    debug!("{}", uid);
    match state.get_ref().get_profile(&uid).await {
        Ok(profile) => HttpResponse::Ok().json(&json!({"message": "", "data": profile})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            db_error(&e, json!("fail"))
        }
    }
}

#[post("/{id}/update-profile")]
async fn update_profile(
    params: web::types::Path<String>,
    form: web::types::Json<ProfileReq>,
    state: AppState,
    auth: auth::AuthorizationService,
) -> impl Responder {
    let uid = match parse_id(&params) {
        Ok(id) => id,
        Err(r) => return r,
    };
    let f = form.into_inner();
    debug!("{}", uid);
    let self_user = verify_profile_user(uid, &auth).await;

    if !self_user {
        return HttpResponse::Unauthorized()
            .json(&json!({"success": false, "message": "Only self can update the profile!"}));
    }

    if let Err(e) = f.validate() {
        return HttpResponse::BadRequest()
            .json(&json!({"success": false, "message": e.to_string()}));
    }

    match state.get_ref().update_profile(&uid, &f).await {
        Ok(_profile) => HttpResponse::Ok().json(&json!({"success": true})),
        Err(e) if is_unique_violation(&e) => HttpResponse::Conflict()
            .json(&json!({"success": false, "message": "That username is taken."})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            db_error(&e, json!(false))
        }
    }
}

#[get("/user/{id}")]
async fn get_user(
    params: web::types::Path<String>,
    auth_guard: auth::AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = match parse_id(&params) {
        Ok(id) => id,
        Err(r) => return r,
    };

    if auth_guard.user.id != uid {
        return HttpResponse::Forbidden()
            .json(&json!({"success": false, "message": "Only self can update user!"}));
    }

    debug!("{}", uid);
    match state.get_ref().get_profile(&uid).await {
        Ok(profile) => HttpResponse::Ok().json(&json!({"message": "", "data": profile})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            db_error(&e, json!("fail"))
        }
    }
}

// #[get("/profile/{id}/username/{username}")]
// async fn update_username(
//     params: web::types::Path<(String, String)>,
//     auth: AuthorizationService,
//     state: AppState,
// ) -> impl Responder {
//     let uid = params.0.parse::<i64>().unwrap();
//     let username = params.1.parse().unwrap();
//     let user = verify_profile_user(uid, &auth).await;
//     if user {
//         match state.get_ref().update_username(uid, &username).await {
//             Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
//             Err(e) => {
//                 debug!("{:?}", e.to_string());
//                 HttpResponse::InternalServerError()
//                     .json(&json!({"status": "fail", "message": e.to_string()}))
//             }
//         }
//     } else {
//         HttpResponse::Unauthorized().finish()
//     }
// }

// #[get("/profile/{id}/title/{title}")]
// async fn update_title(
//     params: web::types::Path<(String, String)>,
//     auth: AuthorizationService,
//     state: AppState,
// ) -> impl Responder {
//     let uid = params.0.parse::<i64>().unwrap();
//     let title = params.1.parse().unwrap();
//     let user = verify_profile_user(uid, &auth).await;
//     if user {
//         match state.get_ref().update_title(uid, &title).await {
//             Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
//             Err(e) => {
//                 debug!("{:?}", e.to_string());
//                 HttpResponse::InternalServerError()
//                     .json(&json!({"status": "fail", "message": e.to_string()}))
//             }
//         }
//     } else {
//         HttpResponse::Unauthorized().finish()
//     }
// }

// #[get("/profile/{id}/name/{name}")]
// async fn update_name(
//     params: web::types::Path<(String, String)>,
//     auth: AuthorizationService,
//     state: AppState,
// ) -> impl Responder {
//     let uid = params.0.parse::<i64>().unwrap();
//     let name = params.1.parse().unwrap();
//     let user = verify_profile_user(uid, &auth).await;
//     if user {
//         match state.get_ref().update_name(uid, &name).await {
//             Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
//             Err(e) => {
//                 debug!("{:?}", e.to_string());
//                 HttpResponse::InternalServerError()
//                     .json(&json!({"status": "fail", "message": e.to_string()}))
//             }
//         }
//     } else {
//         HttpResponse::Unauthorized().finish()
//     }
// }

// #[get("/profile/{id}/designation/{designation}")]
// async fn update_designation(
//     params: web::types::Path<(String, String)>,
//     auth: AuthorizationService,
//     state: AppState,
// ) -> impl Responder {
//     let uid = params.0.parse::<i64>().unwrap();
//     let designation = params.1.parse().unwrap();
//     let user = verify_profile_user(uid, &auth).await;
//     if user {
//         match state.get_ref().update_designation(uid, &designation).await {
//             Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
//             Err(e) => {
//                 debug!("{:?}", e.to_string());
//                 HttpResponse::InternalServerError()
//                     .json(&json!({"status": "fail", "message": e.to_string()}))
//             }
//         }
//     } else {
//         HttpResponse::Unauthorized().finish()
//     }
// }

// #[get("/profile/{id}/location/{location}")]
// async fn update_location(
//     params: web::types::Path<(String, String)>,
//     auth: AuthorizationService,
//     state: AppState,
// ) -> impl Responder {
//     let uid = params.0.parse::<i64>().unwrap();
//     let location = params.1.parse().unwrap();
//     let user = verify_profile_user(uid, &auth).await;
//     if user {
//         match state.get_ref().update_location(uid, &location).await {
//             Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
//             Err(e) => {
//                 debug!("{:?}", e.to_string());
//                 HttpResponse::InternalServerError()
//                     .json(&json!({"status": "fail", "message": e.to_string()}))
//             }
//         }
//     } else {
//         HttpResponse::Unauthorized().finish()
//     }
// }

// #[get("/edit-links/{uid}")]
// async fn get_links(
//     params: web::types::Path<String>,
//     auth: AuthorizationService,
//     state: AppState,
// ) -> impl Responder {
//     let uid = params.parse::<i64>().unwrap();
//     let user = verify_profile_user(uid, &auth).await;
//     if user {
//         match state.get_ref().get_links(uid).await {
//             Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
//             Err(e) => {
//                 debug!("{:?}", e.to_string());
//                 HttpResponse::InternalServerError()
//                     .json(&json!({"status": "fail", "message": e.to_string()}))
//             }
//         }
//     } else {
//         HttpResponse::Unauthorized().finish()
//     }
// }

// #[post("/edit-links/{uid}")]
// async fn update_links(
//     params: web::types::Path<String>,
//     form: web::types::Json<LinksResponse>,
//     auth: AuthorizationService,
//     state: AppState,
// ) -> impl Responder {
//     let uid = params.parse::<i64>().unwrap();
//     let data = form.into_inner();
//     let user = verify_profile_user(uid, &auth).await;
//     if user {
//         match state.get_ref().update_links(uid, &data).await {
//             Ok(r) => HttpResponse::Ok().json(&json!({"message": "", "data": r})),
//             Err(e) => {
//                 debug!("{:?}", e.to_string());
//                 HttpResponse::InternalServerError()
//                     .json(&json!({"status": "fail", "message": e.to_string()}))
//             }
//         }
//     } else {
//         HttpResponse::Unauthorized().finish()
//     }
// }

#[post("/{id}/profile-image-upload")]
async fn image_upload(
    params: web::types::Path<String>,
    payload: Multipart,
    state: AppState,
    auth: AuthorizationService,
) -> impl Responder {
    let uid = match parse_id(&params) {
        Ok(id) => id,
        Err(r) => return r,
    };
    let self_user = verify_profile_user(uid, &auth).await;

    if !self_user {
        return HttpResponse::Unauthorized().json(
            &json!({"success": false, "message": "Only self can update the profile image!"}),
        );
    }

    let url = match save_upload(payload, state.get_ref()).await {
        Ok(url) => url,
        Err(r) => return r,
    };

    match state.get_ref().update_profile_image(uid, &url).await {
        Ok(_r) => HttpResponse::Ok().json(&json!({"success": true, "url": url})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            db_error(&e, json!(false))
        }
    }
}

#[get("/{id}/summary")]
async fn user_summary(params: web::types::Path<String>, state: AppState) -> impl Responder {
    let uid = match parse_id(&params) {
        Ok(id) => id,
        Err(r) => return r,
    };

    match state.get_ref().get_summary(uid).await {
        Ok(r) => HttpResponse::Ok().json(&json!({"success": true, "data": r})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            db_error(&e, json!(false))
        }
    }
}

#[post("/search-users")]
async fn search_users(
    form: web::types::Json<SearchUserRequest>,
    state: AppState,
) -> impl Responder {
    let user_req = form.into_inner();
    // The page size comes from the client: keep it within the configured
    // page size (a negative LIMIT is a database error, a huge one a dump).
    let per_page = user_req
        .users_per_page
        .clamp(1, (state.config.users_per_page as i64).max(1));
    match state
        .get_ref()
        .search_users(&user_req.username, per_page)
        .await
    {
        Ok(u) => HttpResponse::Ok().json(&json!({"data": u})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            db_error(&e, json!("fail"))
        }
    }
}

#[post("/delete-profile/{id}")]
async fn delete_profile(
    params: web::types::Path<String>,
    auth: auth::AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = match parse_id(&params) {
        Ok(id) => id,
        Err(r) => return r,
    };

    info!("{} {}", uid, auth.user.id);

    if uid != auth.user.id {
        return HttpResponse::Unauthorized()
            .json(&json!({"success": false, "message": "Only self can delete self's profile!"}));
    }

    match state.get_ref().delete_profile(uid).await {
        Ok(_u) => HttpResponse::Ok().json(&json!({"success": true})),
        Err(e) => {
            debug!("update tag info: {:?}", e);
            db_error(&e, json!("fail"))
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
    cfg.service(refresh_access_token_handler);
    cfg.service(logout_handler);
    cfg.service(register);
    cfg.service(check_username_availability);
    cfg.service(confirm_email);
    cfg.service(get_users);
    cfg.service(get_profile);
    // cfg.service(update_username);
    // cfg.service(update_title);
    // cfg.service(update_name);
    // cfg.service(update_designation);
    // cfg.service(update_location);
    // cfg.service(get_links);
    // cfg.service(update_links);
    cfg.service(get_user);
    cfg.service(image_upload);
    cfg.service(update_profile);
    cfg.service(user_summary);
    cfg.service(resend_confirmation_email);
    cfg.service(search_users);
    cfg.service(delete_profile);
    cfg.service(submit_email_for_login);
}

#[cfg(test)]
mod tests {
    use super::cookie_domain;

    /// RFC 6265 forbids a port in `Domain`. The shipped `template.json` has
    /// `host: "localhost:5173"`, and browsers silently drop the whole session
    /// cookie when the attribute carries one.
    #[test]
    fn cookie_domain_strips_the_port() {
        assert_eq!(cookie_domain("localhost:5173"), "localhost");
        assert_eq!(cookie_domain("localhost:3000"), "localhost");
        assert_eq!(
            cookie_domain("kunjika.example.com:8080"),
            "kunjika.example.com"
        );
    }

    #[test]
    fn cookie_domain_keeps_a_bare_host() {
        assert_eq!(cookie_domain("localhost"), "localhost");
        assert_eq!(cookie_domain("kunjika.example.com"), "kunjika.example.com");
    }

    /// A misconfigured/empty `host` must not panic the request handler, and
    /// surrounding whitespace must not leak into the attribute.
    #[test]
    fn cookie_domain_is_total() {
        assert_eq!(cookie_domain(""), "");
        assert_eq!(cookie_domain(" localhost "), "localhost");
        assert_eq!(cookie_domain("localhost :5173"), "localhost");
    }
}

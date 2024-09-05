use super::dao::IUser;
use super::user::*;
use crate::middlewares::auth::{self, AuthorizationService};
use crate::state::AppState;
use crate::users::token;
use crate::utils::security::{check_signature, sign};
use crate::utils::verify_user::verify_profile_user;

use cookie::time::Duration;
use cookie::Cookie;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use mobc_redis::redis::{self, AsyncCommands};
use ntex::http::HttpMessage;
use ntex::web::{self, get, post, Error, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use validator::Validate;

// curl -v --data '{"name": "Bob", "email": "Bob@google.com", "password": "Bobpass"}' -H "Content-Type: application/json" -X POST localhost:8080/user/register
#[post("/register")]
async fn register(form: web::types::Json<Register>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    if let Err(e) = form.validate() {
        error!("regitser {:?} error: {:?}", form, e);
        return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": e.to_string()}));
    }
    if !form.match_password() {
        return HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": "Passwords are bad!"}));
    }
    match state.get_ref().user_add(&form).await {
        Ok(res) => {
            debug!("register {:?} res: {}", form, res);
            // TODO: move it out to a common logic
            let _smtp_credentials = Credentials::new(
                state.config.mail_username.clone(),
                state.config.mail_password.clone(),
            );

            let mailer =
                        // For production
                        // AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&state.config.mail_host)
                            //     .unwrap()
                            //     .credentials(smtp_credentials)
                            //     .build();
                        // For development
                        AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost()               ;

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
                    HttpResponse::InternalServerError().json(
                        &json!({"status": "fail", "message": "Something went wrong, try again!"}),
                    )
                }
            }
        }
        Err(e) => {
            error!("regitser {:?} error: {:?}", form, e);
            HttpResponse::Conflict()
                .json(&json!({"status": "fail", "message": "Either email or username is taken"}))
        }
    }
}

// curl -v --data '{"name": "Bob", "email": "Bob@google.com", "password": "Bobpass"}' -H "Content-Type: application/json" -X POST localhost:8080/user/login
#[post("/auth/login")]
async fn login(form: web::types::Json<Login>, state: AppState) -> impl Responder {
    let form = form.into_inner();

    // todo: distable login for deleted and blocked users
    match state.get_ref().user_query(&form.email).await {
        Ok(user) => {
            info!("find user {:?} ok: {:?}", form, user);

            if form.verify(&user.password_hash) {
                let access_token_details = match token::generate_jwt_token(
                    user.id,
                    state.config.access_token_max_age,
                    state.config.access_token_private_key.to_owned(),
                ) {
                    Ok(token_details) => token_details,
                    Err(e) => {
                        return HttpResponse::BadGateway()
                            .json(&json!({"status": "fail", "message": format_args!("{}", e)}));
                    }
                };

                info!("Access token built");
                let refresh_token_details = match token::generate_jwt_token(
                    user.id,
                    state.config.refresh_token_max_age,
                    state.config.refresh_token_private_key.to_owned(),
                ) {
                    Ok(token_details) => token_details,
                    Err(e) => {
                        return HttpResponse::BadGateway()
                            .json(&json!({"status": "fail", "message": format_args!("{}", e)}));
                    }
                };

                let mut redis_client = match state.kv.get().await {
                    Ok(redis_client) => redis_client,
                    Err(_e) => {
                        // debug!("{}",             format_args!(e));
                        return HttpResponse::InternalServerError().json(
                            &json!({"status": "fail", "message": "An internal server occurred!"}),
                        );
                    }
                };

                let access_result: redis::RedisResult<()> = redis_client
                    .set_ex(
                        access_token_details.token_uuid.to_string(),
                        user.id.to_string(),
                        (state.config.access_token_max_age * 60) as usize,
                    )
                    .await;

                if let Err(e) = access_result {
                    return HttpResponse::UnprocessableEntity()
                        .json(&json!({"status": "error", "message": format_args!("{}", e)}));
                }

                let refresh_result: redis::RedisResult<()> = redis_client
                    .set_ex(
                        refresh_token_details.token_uuid.to_string(),
                        user.id.to_string(),
                        (state.config.refresh_token_max_age * 60) as usize,
                    )
                    .await;

                if let Err(e) = refresh_result {
                    return HttpResponse::UnprocessableEntity()
                        .json(&json!({"status": "error", "message": format_args!("{}", e)}));
                }

                drop(redis_client);

                let access_cookie =
                    Cookie::build(("access_token", access_token_details.token.clone().unwrap()))
                        .domain(&state.config.host)
                        .path("/")
                        .secure(true)
                        .max_age(Duration::new(state.config.access_token_max_age * 60, 0))
                        .http_only(true);
                let refresh_cookie =
                    Cookie::build(("refresh_token", refresh_token_details.token.unwrap()))
                        .domain(&state.config.host)
                        .path("/")
                        .secure(true)
                        .max_age(Duration::new(state.config.refresh_token_max_age * 60, 0))
                        .http_only(true);
                let xsrf_cookie =
                    Cookie::build(("xsrf_token", access_token_details.token_uuid.to_string()))
                        .domain(&state.config.host)
                        .path("/")
                        .secure(true)
                        .max_age(Duration::new(state.config.access_token_max_age * 60, 0))
                        .http_only(false);

                let r: LoginResponse = LoginResponse {
                    user,
                    success: true,
                };
                let resp = match serde_json::to_string(&r) {
                    Ok(json) => HttpResponse::Ok()
                        .cookie(access_cookie.to_string())
                        .cookie(refresh_cookie.to_string())
                        .cookie(xsrf_cookie.to_string())
                        .cookie(
                            Cookie::build(("logged_in", json))
                                .domain(&state.config.host)
                                .path("/")
                                .secure(true)
                                .http_only(true)
                                .max_age(Duration::new(state.config.access_token_max_age * 60, 0))
                                .to_string(),
                        )
                        .content_type("application/json")
                        .body(""),
                    Err(e) => Error::from(e).into(),
                };
                resp
            } else {
                HttpResponse::Unauthorized()
                    .json(&json!({"message": "Username or password is wrong!"}))
            }
        }
        Err(e) => {
            debug!("find user {:?} error: {:?}", form, e);
            HttpResponse::Unauthorized().finish()
        }
    }
}

#[get("/auth/refresh")]
async fn refresh_access_token_handler(req: HttpRequest, state: AppState) -> impl Responder {
    let message = "could not refresh access token";

    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => {
            info!("step 1");
            return HttpResponse::Forbidden().json(&json!({"status": "fail", "message": message}));
        }
    };

    let refresh_token_details = match token::verify_jwt_token(
        state.config.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            info!("step 2");
            return HttpResponse::Forbidden()
                .json(&json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let result = state.kv.get().await;
    let mut redis_client = match result {
        Ok(redis_client) => redis_client,
        Err(e) => {
            info!("step 3");
            return HttpResponse::Forbidden().json(
                &json!({"status": "fail", "message": format!("Could not connect to Redis: {}", e)}),
            );
        }
    };
    let redis_result: redis::RedisResult<String> = redis_client
        .get(refresh_token_details.token_uuid.to_string())
        .await;

    let user_id = match redis_result {
        Ok(value) => value,
        Err(e) => {
            info!("step 4");
            return HttpResponse::Forbidden()
                .json(&json!({"status": "fail", "message": e.to_string()}));
        }
    };

    let user_id = user_id.parse::<i64>().unwrap();
    let query_result = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, created_date, image_url, email_verified,
        modified_date, status, designation, git, location, website FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.sql)
    .await
    .unwrap();

    if query_result.is_none() {
        return HttpResponse::Forbidden()
            .json(&json!({"status": "fail", "message": "the user belonging to this token no logger exists"}));
    }

    let user = query_result.unwrap();

    let access_token_details = match token::generate_jwt_token(
        user.id,
        state.config.access_token_max_age,
        state.config.access_token_private_key.to_owned(),
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(&json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let refresh_token_details = match token::generate_jwt_token(
        user.id,
        state.config.refresh_token_max_age,
        state.config.refresh_token_private_key.to_owned(),
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(&json!({"status": "fail", "message": format_args!("{}", e)}));
        }
    };

    let redis_result: redis::RedisResult<()> = redis_client
        .set_ex(
            refresh_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (state.config.access_token_max_age * 60) as usize,
        )
        .await;

    if redis_result.is_err() {
        return HttpResponse::UnprocessableEntity().json(
            &json!({"status": "error", "message": format_args!("{:?}", redis_result.unwrap_err())}),
        );
    }

    let redis_result: redis::RedisResult<()> = redis_client
        .set_ex(
            access_token_details.token_uuid.to_string(),
            user.id.to_string(),
            (state.config.access_token_max_age * 60) as usize,
        )
        .await;

    if redis_result.is_err() {
        return HttpResponse::UnprocessableEntity().json(
            &json!({"status": "error", "message": format_args!("{:?}", redis_result.unwrap_err())}),
        );
    }

    drop(redis_client);

    let access_cookie =
        Cookie::build(("access_token", access_token_details.token.clone().unwrap()))
            .domain(&state.config.host)
            .path("/")
            .secure(true)
            .max_age(Duration::new(state.config.access_token_max_age * 60, 0))
            .http_only(true);

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token_details.token.unwrap()))
        .domain(&state.config.host)
        .path("/")
        .secure(true)
        .max_age(Duration::new(state.config.refresh_token_max_age * 60, 0))
        .http_only(true);

    let xsrf_cookie = Cookie::build(("xsrf_token", access_token_details.token_uuid.to_string()))
        .domain(&state.config.host)
        .path("/")
        .secure(true)
        .max_age(Duration::new(state.config.access_token_max_age * 60, 0))
        .http_only(false);

    let r: LoginResponse = LoginResponse {
        user,
        success: true,
    };
    let resp = match serde_json::to_string(&r) {
        Ok(json) => HttpResponse::Ok()
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .header("Pragma", "no-cache")
            .header("Expires", 0)
            .cookie(access_cookie.to_string())
            .cookie(refresh_cookie.to_string())
            .cookie(xsrf_cookie.to_string())
            .cookie(
                Cookie::build(("logged_in", json))
                    .domain(&state.config.host)
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .max_age(Duration::new(state.config.access_token_max_age * 60, 0))
                    .to_string(),
            )
            .content_type("application/json")
            .body(""),
        Err(e) => Error::from(e).into(),
    };
    resp
}

#[post("/auth/logout")]
async fn logout_handler(
    req: HttpRequest,
    auth_guard: auth::AuthorizationService,
    state: AppState,
) -> impl Responder {
    let message = "Token is invalid or session has expired";

    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden().json(&json!({"status": "fail", "message": message}));
        }
    };

    let refresh_token_details = match token::verify_jwt_token(
        state.config.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(&json!({"status": "fail", "message": format_args!("{:?}", e)}));
        }
    };

    let mut redis_client = state.kv.get().await.unwrap();
    let redis_result: redis::RedisResult<usize> = redis_client
        .del(&[
            refresh_token_details.token_uuid.to_string(),
            auth_guard.xsrf_token,
        ])
        .await;

    if redis_result.is_err() {
        return HttpResponse::UnprocessableEntity().json(
            &json!({"status": "fail", "message": format_args!("{:?}", redis_result.unwrap_err())}),
        );
    }

    drop(redis_client);

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
        .cookie(access_cookie.to_string())
        .cookie(refresh_cookie.to_string())
        .cookie(logged_in_cookie.to_string())
        .json(&json!({"status": "success"}))
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
    let email = check_signature(&token, &state).await;
    if &email == "Signature was expired" {
        HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": "Signature has expired!"}))
    } else {
        match state.get_ref().verify_email(&email).await {
            Ok(_user) => {
                debug!("User found, username unavailable");
                HttpResponse::Ok().json(&json!({"message": "username available"}))
            }
            Err(e) => {
                debug!("{:?}", e.to_string());
                HttpResponse::BadRequest().json(
                    &json!({"status": "fail", "message": "Your email is not registered with us!"}),
                )
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
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[get("/user/{id}/{username}")]
async fn get_profile(
    params: web::types::Path<(String, String)>,
    state: AppState,
) -> impl Responder {
    let uid = &params.0.parse::<i64>().unwrap();
    debug!("{}", uid);
    match state.get_ref().get_profile(&uid).await {
        Ok(profile) => HttpResponse::Ok().json(&json!({"message": "", "data": profile})),
        Err(e) => {
            debug!("{:?}", e.to_string());
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
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
                HttpResponse::InternalServerError()
                    .json(&json!({"status": "fail", "message": e.to_string()}))
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
                HttpResponse::InternalServerError()
                    .json(&json!({"status": "fail", "message": e.to_string()}))
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
                HttpResponse::InternalServerError()
                    .json(&json!({"status": "fail", "message": e.to_string()}))
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
                HttpResponse::InternalServerError()
                    .json(&json!({"status": "fail", "message": e.to_string()}))
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
                HttpResponse::InternalServerError()
                    .json(&json!({"status": "fail", "message": e.to_string()}))
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
                HttpResponse::InternalServerError()
                    .json(&json!({"status": "fail", "message": e.to_string()}))
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
                HttpResponse::InternalServerError()
                    .json(&json!({"status": "fail", "message": e.to_string()}))
            }
        }
    } else {
        HttpResponse::Unauthorized().finish()
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
    cfg.service(update_username);
    cfg.service(update_title);
    cfg.service(update_name);
    cfg.service(update_designation);
    cfg.service(update_location);
    cfg.service(get_links);
    cfg.service(update_links);
}

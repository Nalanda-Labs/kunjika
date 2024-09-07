use std::{fs, io::Write};

use super::dao::IQuestion;
use super::question::*;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::users::user::UserCookie;
use crate::utils::slug::create_slug;

use chrono::*;
use futures::{StreamExt, TryStreamExt};
use ntex::{
    http::HttpMessage,
    web::{self, get, post, HttpRequest, HttpResponse, Responder},
};
use ntex_multipart::Multipart;
use serde_json::json;
use uuid::Uuid;

#[post("/create-question")]
async fn insert_question(
    form: ntex::web::types::Json<QuestionRequest>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let r = form.into_inner();
    if r.tag_list.len() == 0 {
        HttpResponse::UnprocessableEntity()
            .json(&json!({"status": "fail", "message": "All tags were not found"}));
    }

    let slug = create_slug(&r.title).await;

    let q = DbQuestion {
        title: r.title,
        description: r.description,
        posted_by_id: auth.user.id,
        slug: slug.clone(),
        updated_by_id: auth.user.id,
        tag_list: r.tag_list,
    };

    match state.get_ref().insert_question(&q).await {
        Ok(0) => {
            return HttpResponse::BadRequest()
                .json(&json!({"status": "fail", "message": "All tags were not found"}))
        }
        Ok(t) => {
            debug!("insert question {:?} ", t);
            let res = AskResponse {
                id: t.to_string(),
                slug,
            };
            HttpResponse::Ok().json(&json!({"data": res}))
        }
        Err(e) => {
            error!("insert question error: {:?}", e);
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
        }
    }
}

#[get("/questions/{id}/{slug}")]
async fn get_question(
    params: ntex::web::types::Path<(String, String)>,
    req: HttpRequest,
    state: AppState,
) -> impl Responder {
    let remote_address = match req.peer_addr() {
        Some(ra) => ra.to_string(),
        None => "".to_owned(),
    };

    let parts: Vec<&str> = remote_address.split(":").collect();
    let mut ipaddr = "";

    if parts[0] != "" {
        ipaddr = parts[0];
    }

    let logged_in = match req.cookie("logged_in") {
        Some(s) => s.to_string(),
        None => "".to_owned(),
    };

    debug!("{:?}", logged_in);
    let cookie_str: Vec<&str> = logged_in.split("=").collect();
    let u: UserCookie;
    let mut uid = 0;

    if cookie_str.len() != 2 {
        // the user is not logged in
    } else {
        u = serde_json::from_str(cookie_str[1]).unwrap();
        uid = u.user.id;
    }

    let qid: i64 = (params.0).parse().unwrap();
    debug!("Question id is {}", qid);
    match state.get_ref().get_question(qid, uid, ipaddr).await {
        Ok(db_question) => HttpResponse::Ok().json(&json!({"data": db_question})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/questions/")]
async fn get_questions(
    ut: ntex::web::types::Json<QuestionsReq>,
    state: AppState,
) -> impl Responder {
    let updated_at = ut.into_inner();
    let up_at;
    debug!("{:?}", &updated_at.updated_at);
    if updated_at.updated_at == "" {
        up_at = chrono::offset::Utc::now();
        debug!("{:?}", up_at);
    } else {
        up_at = chrono::DateTime::parse_from_rfc3339(&updated_at.updated_at)
            .unwrap()
            .with_timezone(&Utc);
    }
    match state.get_ref().get_questions(&up_at).await {
        Ok(db_questions) => HttpResponse::Ok().json(&json!({"data": db_questions})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/questions/tagged/{tag}")]
async fn get_questions_by_tag(
    params: ntex::web::types::Path<String>,
    ut: ntex::web::types::Json<QuestionsReq>,
    state: AppState,
) -> impl Responder {
    let tag = params.parse().unwrap();
    let updated_at = ut.into_inner();
    let up_at;
    debug!("{:?}", &updated_at.updated_at);
    if updated_at.updated_at == "" {
        up_at = chrono::offset::Utc::now();
        debug!("{:?}", up_at);
    } else {
        up_at = chrono::DateTime::parse_from_rfc3339(&updated_at.updated_at)
            .unwrap()
            .with_timezone(&Utc);
    }
    match state.get_ref().get_questions_by_tag(&up_at, &tag).await {
        Ok(db_questions) => HttpResponse::Ok().json(&json!({"data": db_questions})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[get("/question/get-answers/{id}/")]
async fn get_answers(
    params: ntex::web::types::Path<i64>,
    q: ntex::web::types::Query<AnswersQuery>,
    state: AppState,
) -> impl Responder {
    let qid = params.into_inner();
    debug!("{:?}, {:?}, {:?}", &qid, q.time, q.limit);

    match state.get_ref().get_answers(qid, &q.time, q.limit).await {
        Ok(db_answers) => HttpResponse::Ok().json(&json!({"data": db_answers})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/answer")]
async fn answer(
    params: ntex::web::types::Json<AnswerReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let answer = params.into_inner();
    debug!(
        "answer request is {:?}, {:?}, {:?}",
        &answer.id, &answer.value, &answer.reply_to
    );

    match state.get_ref().insert_answer(&answer, &auth.user.id).await {
        Ok(answer_res) => HttpResponse::Ok().json(&json!({"data": answer_res.to_string()})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[get("edit/question/{id}")]
async fn get_edit_post(
    params: ntex::web::types::Path<i64>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let pid = params.into_inner();

    debug!("post id to be edited is {:?}", pid);

    match state.get_ref().get_post(pid).await {
        Ok(post) => HttpResponse::Ok().json(&json!({"data": post})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/edit-post/{id}")]
async fn update_post(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<EditRequest>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let pid = params.into_inner();
    let er = form.into_inner();
    let title = match er.title {
        Some(t) => t,
        None => "".to_owned(),
    };
    let slug = create_slug(&title.to_string()).await;

    match state
        .get_ref()
        .update_post(pid, &er.description, &er.tag_list, &title, &slug)
        .await
    {
        Ok((post, slug)) => HttpResponse::Ok().json(&json!({"data": { "id": post, "slug": slug}})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/image-upload")]
async fn image_upload(mut payload: Multipart, state: AppState) -> impl Responder {
    // This will panic in case of error, which is a good thing because it means it is broken.
    // It must not panic in general for the app to be usable.
    // we do not check for image size. This should be set at web server level
    // by limiting client max body size.
    let current_date = chrono::Utc::now();
    let year = current_date.year();
    let month = current_date.month();
    let day = current_date.day();
    // relative path from root
    let path = format!("{}/{}/{}/{}", state.config.upload_folder, year, month, day);
    fs::create_dir_all(&path).unwrap();
    let filename = Uuid::new_v4().to_string();
    let filepath = format!("{}/{}", &path, filename);

    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        // File::create is blocking operation, use threadpool
        // TODO: fix this
        let filepath1 = format!("{}/{}", &path, filename);
        let mut f = web::block(|| std::fs::File::create(filepath1))
            .await
            .unwrap();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f))
                .await
                .unwrap();
        }
    }

    let url = state.config.backend_url.clone() + &filepath;

    return HttpResponse::Ok().json(&json!({"success": "true", "url": url}));
}

pub fn init(cfg: &mut ntex::web::ServiceConfig) {
    cfg.service(insert_question);
    cfg.service(get_question);
    cfg.service(get_questions);
    cfg.service(get_answers);
    cfg.service(answer);
    cfg.service(get_questions_by_tag);
    cfg.service(get_edit_post);
    cfg.service(update_post);
    cfg.service(image_upload);
}

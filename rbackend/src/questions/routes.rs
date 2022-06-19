use super::dao::IQuestion;
use super::question::*;
use crate::api::ApiResult;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::utils::slug::create_slug;

use actix_web::{get, post, web, Responder};
use chrono::*;

#[post("/create-question")]
async fn insert_question(
    form: web::Json<QuestionRequest>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let r = form.into_inner();
    if r.tag_list.len() == 0 {
        return ApiResult::new()
            .code(422)
            .with_msg("At least one tag must be supplied!");
    }

    let slug = create_slug(&r.title).await;

    let q = DbQuestion {
        title: r.title,
        description: r.description,
        posted_by_id: auth.claims.id,
        slug: slug.clone(),
        updated_by_id: auth.claims.id,
        tag_list: r.tag_list,
    };

    match state.get_ref().insert_question(&q).await {
        Ok(0) => {
            ApiResult::new().code(400).with_msg("All tags were not found")
        }
        Ok(t) => {
            debug!("insert question {:?} ", t);
            let res = AskResponse {
                id: t.to_string(),
                slug,
            };

            ApiResult::new().with_msg("ok").with_data(res)
        }
        Err(e) => {
            error!("insert question error: {:?}", e);
            ApiResult::new().code(502).with_msg(e.to_string())
        }
    }
}

#[get("/questions/{id}/{slug}")]
async fn get_question(params: web::Path<(String, String)>, state: AppState) -> impl Responder {
    let qid: i64 = (params.0).parse().unwrap();
    debug!("Question id is {}", qid);
    match state.get_ref().get_question(qid).await {
        Ok(db_question) => ApiResult::new().with_msg("").with_data(db_question),
        Err(e) => ApiResult::new().code(502).with_msg(e.to_string()),
    }
}

#[post("/questions/")]
async fn get_questions(ut: web::Json<QuestionsReq>, state: AppState) -> impl Responder {
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
        Ok(db_questions) => ApiResult::new().with_msg("").with_data(db_questions),
        Err(e) => ApiResult::new().code(502).with_msg(e.to_string()),
    }
}

#[post("/questions/tagged/{tag}")]
async fn get_questions_by_tag(
    params: web::Path<String>,
    ut: web::Json<QuestionsReq>,
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
        Ok(db_questions) => ApiResult::new().with_msg("").with_data(db_questions),
        Err(e) => ApiResult::new().code(502).with_msg(e.to_string()),
    }
}

#[get("/question/get-answers/{id}/")]
async fn get_answers(
    params: web::Path<i64>,
    q: web::Query<AnswersQuery>,
    state: AppState,
) -> impl Responder {
    let qid = params.into_inner();
    debug!("{:?}, {:?}, {:?}", &qid, q.time, q.limit);

    match state.get_ref().get_answers(qid, &q.time, q.limit).await {
        Ok(db_answers) => ApiResult::new().with_msg("").with_data(db_answers),
        Err(e) => ApiResult::new().code(502).with_msg(e.to_string()),
    }
}

#[post("/answer")]
async fn answer(
    params: web::Json<AnswerReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let answer = params.into_inner();
    debug!(
        "answer request is {:?}, {:?}, {:?}",
        &answer.id, &answer.value, &answer.reply_to
    );

    match state
        .get_ref()
        .insert_answer(&answer, &auth.claims.id)
        .await
    {
        Ok(answer_res) => ApiResult::new()
            .with_msg("")
            .with_data(answer_res.to_string()),
        Err(e) => ApiResult::new().code(502).with_msg(e.to_string()),
    }
}

#[get("edit/question/{id}")]
async fn get_edit_post(
    params: web::Path<i64>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let pid = params.into_inner();

    debug!("post id to be edited is {:?}", pid);

    match state.get_ref().get_post(pid).await {
        Ok(post) => ApiResult::new().with_msg("").with_data(post),
        Err(e) => ApiResult::new().code(502).with_msg(e.to_string()),
    }
}

#[post("/edit-post/{id}")]
async fn update_post(
    params: web::Path<i64>,
    form: web::Json<EditRequest>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let pid = params.into_inner();
    let er = form.into_inner();
    let title = match er.title {
        Some(t) => t,
        None => "".to_owned()
    };
    let slug = create_slug(&title.to_string()).await;

    match state.get_ref().update_post(pid, &er.description, &er.tag_list, &title, &slug).await {
        Ok(post) => ApiResult::new().code(200).with_msg("".to_owned()).with_data(post),
        Err(e) => ApiResult::new().code(502).with_msg(e.to_string()).with_data(0)
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(insert_question);
    cfg.service(get_question);
    cfg.service(get_questions);
    cfg.service(get_answers);
    cfg.service(answer);
    cfg.service(get_questions_by_tag);
    cfg.service(get_edit_post);
    cfg.service(update_post);
}

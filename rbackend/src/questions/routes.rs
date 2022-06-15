use super::dao::IQuestion;
use super::question::*;
use crate::api::ApiResult;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

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

    let mut slug = "".to_string();
    let mut prev_dash = false;
    for c in r.title.chars() {
        if (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') {
            slug.push(c);
            prev_dash = false;
        } else if c == ' ' || c == ',' || c == '.' || c == '/' || c == '\\' || c == '_' || c == '='
        {
            if (!prev_dash) && (slug.len() > 0) {
                slug.push('-');
                prev_dash = true;
            }
        }
    }
    if prev_dash {
        slug = (&slug[0..slug.len() - 1]).to_string();
    }

    let q = DbQuestion {
        title: r.title,
        description: r.description,
        posted_by_id: auth.claims.id,
        slug: slug.clone(),
        updated_by_id: auth.claims.id,
        tag_list: r.tag_list,
    };

    match state.get_ref().insert_question(&q).await {
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
    debug!("{:?}, {:?}, {:?}", &answer.id, &answer.value, &answer.reply_to);

    match state.get_ref().insert_answer(&answer, &auth.claims.id).await {
        Ok(answer_res) => ApiResult::new().with_msg("").with_data(answer_res.to_string()),
        Err(e) => ApiResult::new().code(502).with_msg(e.to_string()),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(insert_question);
    cfg.service(get_question);
    cfg.service(get_questions);
    cfg.service(get_answers);
    cfg.service(answer);
}

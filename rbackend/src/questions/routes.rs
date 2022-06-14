use super::dao::IQuestion;
use super::question::*;
use crate::api::ApiResult;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;

use actix_web::{post, web, Responder};

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
        op_id: auth.claims.id,
        posted_by_id: auth.claims.id,
        slug: slug.clone(),
        updated_by_id: auth.claims.id,
        tag_list: r.tag_list
    };

    match state.get_ref().insert_question(&q).await {
        Ok(t) => {
            debug!("insert question {:?} ", t);
            let res = AskResponse { id:  t.to_string(), slug};

            return ApiResult::new().with_msg("ok").with_data(res);
        }
        Err(e) => {
            error!("insert question error: {:?}", e);
            return ApiResult::new().code(500).with_msg(e.to_string());
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(insert_question);
}

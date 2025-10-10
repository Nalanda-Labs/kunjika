use ntex::web::{self, HttpResponse, Responder};
use ntex::web::types::Query;
use serde::Deserialize;

use crate::state::AppState;
use crate::search::quickwit::Quickwit;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/search").route(web::get().to(search)));
}

#[derive(Deserialize)]
struct QueryParams {
    q: String,
    limit: Option<usize>,
}

async fn search(q: Query<QueryParams>, state: AppState) -> impl Responder {
    let q = q.into_inner();

    if let Some(qw) = Quickwit::from_config(&state.config) {
        match qw.search(&q.q, q.limit.unwrap_or(20)).await {
            Ok(v) => HttpResponse::Ok().json(&v), // pass by reference
            Err(e) => HttpResponse::InternalServerError().json(&serde_json::json!({
                "status":"fail","message":format!("search error: {}", e)
            })),
        }
    } else {
        HttpResponse::BadRequest().json(&serde_json::json!({
            "status":"fail","message":"Quickwit not configured"
        }))
    }
}

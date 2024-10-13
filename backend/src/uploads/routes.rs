use std::fs;

use crate::state::AppState;
use ntex::web::{self, get, HttpResponse, Responder};

#[get("/uploads/{year}/{month}/{day}/{image_id}")]
async fn get_image(
    path: web::types::Path<(String, String, String, String)>,
    state: AppState,
) -> impl Responder {
    let (year, month, day, image_id) = path.into_inner();
    let path = state.config.upload_folder.clone()
        + "/"
        + &year
        + "/"
        + &month
        + "/"
        + &day
        + "/"
        + &image_id;

    error!("{}", path);

    match web::block(|| fs::read(path)).await {
        Ok(d) => {
            return HttpResponse::Ok().body(d);
        }
        Err(_e) => {
            error!("{}", _e.to_string());
            return HttpResponse::NotFound().into();
        }
    };
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_image);
}

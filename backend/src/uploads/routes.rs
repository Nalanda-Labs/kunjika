use std::fs;

use super::save::image_type;
use crate::state::AppState;
use ntex::web::{self, get, HttpResponse, Responder};

/// A path segment is safe when it is non-empty and made only of ASCII
/// letters, digits and '-'. That rules out '.', '/', '\\' and any other
/// character that could walk out of the upload folder.
fn is_safe_segment(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[get("/uploads/{year}/{month}/{day}/{image_id}")]
async fn get_image(
    path: web::types::Path<(String, String, String, String)>,
    state: AppState,
) -> impl Responder {
    let (year, month, day, image_id) = path.into_inner();

    // Uploads are stored as <folder>/<year>/<Month>/<day>/<uuid> (see
    // uploads::save). Anything else - in particular "..", which used to be
    // joined into the path verbatim - is rejected before touching the disk.
    let valid = year.chars().all(|c| c.is_ascii_digit())
        && month.chars().all(|c| c.is_ascii_alphabetic())
        && day.chars().all(|c| c.is_ascii_digit())
        && [&year, &month, &day, &image_id].iter().all(|s| is_safe_segment(s))
        && uuid::Uuid::parse_str(&image_id).is_ok();
    if !valid {
        return HttpResponse::NotFound().into();
    }

    let path = format!(
        "{}/{}/{}/{}/{}",
        state.config.upload_folder, year, month, day, image_id
    );

    info!("{}", path);

    match web::block(|| fs::read(path)).await {
        // Serve only real images, with their real type, and forbid the
        // browser from sniffing or running anything. Files stored before
        // uploads were validated get the same check here.
        Ok(d) => match image_type(&d) {
            Some(mime) => HttpResponse::Ok()
                .content_type(mime)
                .header("X-Content-Type-Options", "nosniff")
                .header("Content-Security-Policy", "default-src 'none'; sandbox")
                .header("Cache-Control", "public, max-age=31536000, immutable")
                .body(d),
            None => HttpResponse::NotFound().into(),
        },
        Err(_e) => {
            error!("{}", _e.to_string());
            HttpResponse::NotFound().into()
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_image);
}

#[cfg(test)]
mod tests {
    use super::is_safe_segment;

    #[test]
    fn rejects_traversal_segments() {
        assert!(!is_safe_segment(""));
        assert!(!is_safe_segment(".."));
        assert!(!is_safe_segment("a/b"));
        assert!(!is_safe_segment("a\\b"));
        assert!(is_safe_segment("November"));
        assert!(is_safe_segment("0f8fad5b-d9cb-469f-a165-70867728950e"));
    }
}

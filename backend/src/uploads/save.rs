use std::io::Write;

use futures::{StreamExt, TryStreamExt};
use ntex::web::{self, HttpResponse};
use ntex_multipart::Multipart;
use serde_json::json;
use uuid::Uuid;

use crate::state::AppStateRaw;
use crate::utils::request::{bad_request, internal_error};

/// Bytes needed to recognise every accepted format (WebP needs 12).
const SNIFF_LEN: usize = 12;

/// The MIME type of a raster image, recognised by its magic bytes, or None.
///
/// Only formats that cannot carry script are accepted - notably not SVG or
/// HTML. Uploads are served from the API origin, which the frontend calls
/// with credentials, so a file a browser would render as a document there
/// is stored XSS.
pub fn image_type(head: &[u8]) -> Option<&'static str> {
    if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if head.starts_with(b"\xff\xd8\xff") {
        Some("image/jpeg")
    } else if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

async fn remove(path: String) {
    let _ = web::block(move || std::fs::remove_file(path)).await;
}

/// Stream the first file in a multipart upload to
/// `<upload_folder>/<year>/<Month>/<day>/<uuid>` and return its public URL
/// (`<backend_url>uploads/<year>/<Month>/<day>/<uuid>`, served by
/// `uploads::routes::get_image`). Enforces `image_max_size` while streaming,
/// accepts only PNG/JPEG/GIF/WebP content, and never panics on a broken or
/// aborted client stream.
pub async fn save_upload(mut payload: Multipart, state: &AppStateRaw) -> Result<String, HttpResponse> {
    let now = time::OffsetDateTime::now_utc();
    let rel = format!("{}/{}/{}/{}", now.year(), now.month(), now.day(), Uuid::new_v4());
    let filepath = format!("{}/{}", state.config.upload_folder, rel);
    let dir = filepath[..filepath.rfind('/').unwrap_or(0)].to_string();

    if let Err(e) = web::block(move || std::fs::create_dir_all(dir)).await {
        return Err(internal_error(e));
    }

    // Only the first part is stored; anything after it is ignored rather than
    // overwriting the same file.
    let mut field = match payload.try_next().await {
        Ok(Some(field)) => field,
        Ok(None) => return Err(bad_request("No file uploaded.")),
        Err(e) => {
            info!("upload stream error: {:?}", e);
            return Err(bad_request("Upload failed, try again."));
        }
    };

    let fp = filepath.clone();
    let mut f = match web::block(move || std::fs::File::create(fp)).await {
        Ok(f) => f,
        Err(e) => return Err(internal_error(e)),
    };

    let mut len = 0usize;
    let mut head: Vec<u8> = Vec::with_capacity(SNIFF_LEN);
    while let Some(chunk) = field.next().await {
        let data = match chunk {
            Ok(d) => d,
            Err(e) => {
                info!("upload stream error: {:?}", e);
                remove(filepath).await;
                return Err(bad_request("Upload failed, try again."));
            }
        };
        len += data.len();
        if len > state.config.image_max_size {
            remove(filepath).await;
            return Err(HttpResponse::BadRequest()
                .json(&json!({"success": false, "message": "Image is too large."})));
        }
        if head.len() < SNIFF_LEN {
            let take = (SNIFF_LEN - head.len()).min(data.len());
            head.extend_from_slice(&data[..take]);
        }
        // filesystem operations are blocking, we have to use threadpool
        f = match web::block(move || f.write_all(&data).map(|_| f)).await {
            Ok(f) => f,
            Err(e) => {
                remove(filepath).await;
                return Err(internal_error(e));
            }
        };
    }

    if image_type(&head).is_none() {
        remove(filepath).await;
        return Err(bad_request("Only PNG, JPEG, GIF and WebP images are allowed."));
    }

    Ok(format!("{}uploads/{}", state.config.backend_url, rel))
}

#[cfg(test)]
mod tests {
    use super::image_type;

    #[test]
    fn recognises_images_only() {
        assert_eq!(image_type(b"\x89PNG\r\n\x1a\nxxxx"), Some("image/png"));
        assert_eq!(image_type(b"\xff\xd8\xff\xe0"), Some("image/jpeg"));
        assert_eq!(image_type(b"GIF89a"), Some("image/gif"));
        assert_eq!(image_type(b"RIFF\0\0\0\0WEBP"), Some("image/webp"));
        assert_eq!(image_type(b"<html><script>"), None);
        assert_eq!(image_type(b"<svg xmlns="), None);
        assert_eq!(image_type(b""), None);
    }
}

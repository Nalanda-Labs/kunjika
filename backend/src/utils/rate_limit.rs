//! Fixed-window rate limits backed by Redis.
//!
//! Each limit is a counter `rl:{bucket}:{key}` that is incremented per
//! request and expires `window` seconds after the first one. Limits come
//! from the config file; a limit of 0 switches that check off.
//!
//! The limiter fails OPEN: if Redis cannot be reached the request goes
//! through and the error is logged. A cache outage should not also stop
//! people from posting.

use mobc_redis::redis::AsyncCommands;
use ntex::web::{HttpRequest, HttpResponse};
use serde_json::json;

use crate::state::AppStateRaw;

pub const HOUR: u64 = 3600;

/// Count one request against `bucket`/`key`. `Err` is the 429 to return when
/// more than `max` requests were made in the current window.
pub async fn check(
    state: &AppStateRaw,
    bucket: &str,
    key: &str,
    max: u32,
    window: u64,
    what: &str,
) -> Result<(), HttpResponse> {
    if max == 0 {
        return Ok(());
    }
    let redis_key = format!("rl:{bucket}:{key}");
    let mut redis = match state.kv.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("rate limit {redis_key}: redis unavailable: {:?}", e);
            return Ok(());
        }
    };
    let used: i64 = match redis.incr(&redis_key, 1).await {
        Ok(n) => n,
        Err(e) => {
            error!("rate limit {redis_key}: incr failed: {:?}", e);
            return Ok(());
        }
    };
    // The first request in a window starts its clock. A counter that somehow
    // has no expiry (the process died between INCR and EXPIRE) would block
    // its key forever, so that case is repaired here too.
    let mut ttl: i64 = redis.ttl(&redis_key).await.unwrap_or(-1);
    if used == 1 || ttl < 0 {
        let _: Result<bool, _> = redis.expire(&redis_key, window as i64).await;
        ttl = window as i64;
    }
    if used <= max as i64 {
        return Ok(());
    }
    info!("rate limit hit: {redis_key} ({used}/{max})");
    Err(HttpResponse::TooManyRequests()
        .header("Retry-After", ttl.max(1).to_string())
        .json(&json!({
            "status": "fail",
            "message": format!(
                "Too many {what}. Please try again in {} minute(s).",
                (ttl.max(1) + 59) / 60
            ),
        })))
}

/// The client's IP address, for per-IP limits.
///
/// Behind the bundled nginx config every connection comes from loopback, so
/// in that case the address is taken from `X-Real-IP` / `X-Forwarded-For`,
/// which only a local proxy can have set. A loopback peer WITHOUT such a
/// header cannot be told apart from any other client; `None` is returned and
/// the caller skips the per-IP limit rather than throttling everyone as one.
pub fn client_ip(req: &HttpRequest) -> Option<String> {
    let peer = req.peer_addr()?.ip();
    if !peer.is_loopback() {
        return Some(peer.to_string());
    }
    // `X-Forwarded-For` is a list each proxy appends to, so only its LAST
    // entry was written by our own proxy; earlier ones are whatever the
    // client chose to send.
    let header = |name: &str| {
        req.headers()
            .get(name)
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.rsplit(',').next())
            .map(|v| v.trim().to_string())
            .filter(|v| v.parse::<std::net::IpAddr>().is_ok())
    };
    header("X-Real-IP").or_else(|| header("X-Forwarded-For"))
}

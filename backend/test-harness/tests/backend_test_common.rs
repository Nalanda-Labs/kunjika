// Shared helpers for the additive-only Kunjika backend test harness.
// See tests/api_users.rs header for the no-touch-existing-files contract.

use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static SPAWN_ONCE: AtomicBool = AtomicBool::new(false);

pub fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// A short, unique username (3-char prefix + 7 random hex = 10 chars).
///
/// `Register` allows 3..=33 and `users.username` is `character varying(33)`
/// since migrations/20261001160000_widen_username_to_33.sql. 10 is simply a
/// compact default that keeps fixtures readable; use [`username_of_len`] when a
/// specific width is what is under test.
pub fn unique_username(prefix: &str) -> String {
    let p: String = prefix
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(3)
        .collect();
    let p = if p.is_empty() { "usr".to_string() } else { p };
    // 3 + 7 = 10 chars, always starts with an ASCII letter
    // (validator rejects a leading digit).
    format!("{}{}", p, &uuid::Uuid::new_v4().to_string()[..7])
}

/// A random username of exactly `n` characters.
///
/// `Register` validates `length(min = 3, max = 33)` and, since
/// `migrations/20261001160000_widen_username_to_33.sql`, the `users.username`
/// column is `character varying(33)` too - so `n` up to 33 is accepted.
/// Always starts with an ASCII letter (the validator rejects a leading digit)
/// and never contains '@' (also rejected).
pub fn username_of_len(n: usize) -> String {
    assert!(
        (3..=99).contains(&n),
        "username_of_len({n}) out of fixture range"
    );
    // Start with a letter (the validator rejects a leading digit) and keep
    // appending random hex until there is enough to truncate to exactly `n`.
    // One UUID is only 32 hex chars, so a single append cannot reach n > 33.
    let mut s = String::from("t");
    while s.len() < n + 1 {
        s.push_str(&uuid::Uuid::new_v4().to_string().replace('-', ""));
    }
    s.truncate(n);
    assert_eq!(s.len(), n, "helper must return exactly n characters");
    s
}

pub fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

pub struct TestCtx {
    pub client: reqwest::blocking::Client,
    pub base_url: String,
    pub db_url: String,
    pub mailpit_url: String,
    // Keep the spawned backend alive for the whole test binary run.
    pub _child: std::sync::Arc<std::sync::Mutex<Option<Child>>>,
}

impl TestCtx {
    pub fn new() -> Self {
        let base_url = env_or("KUNJIKA_TEST_BASE_URL", "http://127.0.0.1:8000/api/v1");
        let db_url = env_or(
            "KUNJIKA_TEST_DB_URL",
            "postgres://yugabyte:yugabyte@127.0.1.1:5433/kunjika_test",
        );
        let mailpit_url = env_or("KUNJIKA_TEST_MAILPIT_URL", "http://127.0.0.1:8025");

        let child: std::sync::Arc<std::sync::Mutex<Option<Child>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));

        // Spawn the backend binary once per test process (unless the caller
        // asked to reuse an already-running server).
        if std::env::var("KUNJIKA_TEST_NO_SPAWN").is_err()
            && !SPAWN_ONCE.swap(true, Ordering::SeqCst)
        {
            let repo = repo_root();
            let bin = env_or(
                "KUNJIKA_TEST_BACKEND_BIN",
                &repo.join("backend/target/debug/kunjika").to_string_lossy(),
            );
            let cfg = env_or(
                "KUNJIKA_TEST_CONFIG",
                &repo.join("backend/template.test.json").to_string_lossy(),
            );
            let mut cmd = Command::new(&bin);
            cmd.arg("-c").arg(&cfg).arg("-v").arg("1");
            cmd.current_dir(repo.join("backend"));
            if std::env::var("KUNJIKA_TEST_NO_SHIM").is_err() {
                if let Some(shim) = ensure_smtp_shim() {
                    cmd.env("LD_PRELOAD", &shim);
                    cmd.env(
                        "KUNJIKA_SMTP_REDIRECT_FROM",
                        env_or("KUNJIKA_TEST_SMTP_FROM_PORT", "25"),
                    );
                    cmd.env(
                        "KUNJIKA_SMTP_REDIRECT_TO",
                        env_or("KUNJIKA_TEST_SMTP_PORT", "2525"),
                    );
                }
            }
            match cmd.spawn() {
                Ok(c) => {
                    *child.lock().unwrap() = Some(c);
                    // Give ntex a moment to bind :8000.
                    std::thread::sleep(Duration::from_secs(3));
                }
                Err(e) => eprintln!("could not spawn backend {bin}: {e}"),
            }
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .cookie_store(true)
            .build()
            .expect("reqwest client");
        Self {
            client,
            base_url,
            db_url,
            mailpit_url,
            _child: child,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    pub fn db(&self) -> postgres::Client {
        postgres::Client::connect(&self.db_url, postgres::NoTls)
            .expect("connect to kunjika_test YSQL")
    }

    /// `postgres::Client::query_one` yields a `Row`, not a tuple, so the
    /// tests go through these small typed helpers instead.
    fn to_sql_vec<'a, P: postgres::types::ToSql + Sync>(
        params: &'a [&'a P],
    ) -> Vec<&'a (dyn postgres::types::ToSql + Sync)> {
        params
            .iter()
            .map(|p| *p as &(dyn postgres::types::ToSql + Sync))
            .collect()
    }

    pub fn exec<P: postgres::types::ToSql + Sync>(&self, sql: &str, params: &[&P]) -> usize {
        let ps = Self::to_sql_vec(params);
        self.db().execute(sql, &ps).expect("db execute failed") as usize
    }

    /// Convenience for statements that take no bind parameters (P cannot be
    /// inferred from an empty slice).
    pub fn exec0(&self, sql: &str) -> usize {
        self.db()
            .execute(sql, &[] as &[&(dyn postgres::types::ToSql + Sync)])
            .expect("db execute failed") as usize
    }

    /// Execute a statement and report success as a bool instead of panicking.
    /// Use this when the statement is *expected* to be rejected (e.g. to prove
    /// a UNIQUE constraint actually fires).
    pub fn try_exec<P: postgres::types::ToSql + Sync>(&self, sql: &str, params: &[&P]) -> bool {
        let ps = Self::to_sql_vec(params);
        self.db().execute(sql, &ps).is_ok()
    }

    /// Insert `name` into `tags` only when it is absent.
    ///
    /// `tags.name` is `citext` with **no unique constraint**, so a plain
    /// `INSERT ... ON CONFLICT DO NOTHING` (which needs a conflict target
    /// anyway) happily accumulates duplicates. Duplicates then break
    /// `questions::dao::insert_question`, which bails out with
    /// "All tags were not found" as soon as
    /// `SELECT id FROM tags WHERE name = ANY($1)` returns a different number
    /// of rows than the request had tags.
    pub fn ensure_tag(&self, name: &str) {
        let existing = self.q_i64("SELECT count(*) FROM tags WHERE name = $1", &[&name]);
        if existing > 1 {
            panic!("tag {name} has {existing} rows - dedupe it before running tests");
        }
        if existing == 0 {
            self.exec(
                "INSERT INTO tags (name, info) VALUES ($1, 'e2e test tag')",
                &[&name],
            );
        }
    }

    pub fn q_string<P: postgres::types::ToSql + Sync>(&self, sql: &str, params: &[&P]) -> String {
        let ps = Self::to_sql_vec(params);
        self.db()
            .query_one(sql, &ps)
            .expect("db query_one failed")
            .get(0)
    }

    pub fn q_i64<P: postgres::types::ToSql + Sync>(&self, sql: &str, params: &[&P]) -> i64 {
        let ps = Self::to_sql_vec(params);
        self.db()
            .query_one(sql, &ps)
            .expect("db query_one failed")
            .get(0)
    }

    pub fn q_bool<P: postgres::types::ToSql + Sync>(&self, sql: &str, params: &[&P]) -> bool {
        let ps = Self::to_sql_vec(params);
        self.db()
            .query_one(sql, &ps)
            .expect("db query_one failed")
            .get(0)
    }

    pub fn q_opt_i64<P: postgres::types::ToSql + Sync>(
        &self,
        sql: &str,
        params: &[&P],
    ) -> Option<i64> {
        let ps = Self::to_sql_vec(params);
        self.db()
            .query_one(sql, &ps)
            .expect("db query_one failed")
            .get(0)
    }
}

pub fn unique_email(prefix: &str) -> String {
    format!(
        "{}-{}@example.com",
        prefix,
        &uuid::Uuid::new_v4().to_string()[..8]
    )
}

/// Build (if needed) and return the LD_PRELOAD shim used to redirect the
/// backend's outbound SMTP connect() from port 25 to a rootless-friendly port.
///
/// backend/src/config.rs hard-codes `unencrypted_localhost()` (127.0.0.1:25)
/// and backend/src/main.rs refuses to run as root, so we cannot simply bind
/// :25. See native/smtp_port_redirect.c for the full rationale.
pub fn ensure_smtp_shim() -> Option<std::path::PathBuf> {
    let src = repo_root().join("backend/test-harness/native/smtp_port_redirect.c");
    let so = repo_root().join("backend/test-harness/native/libsmtp_port_redirect.so");
    if !src.exists() {
        return None;
    }
    let needs_build = match (std::fs::metadata(&so), std::fs::metadata(&src)) {
        (Ok(a), Ok(b)) => {
            a.len() != b.len() || {
                let (a, b) = (a.modified().ok(), b.modified().ok());
                match (a, b) {
                    (Some(a), Some(b)) => b > a,
                    _ => true,
                }
            }
        }
        _ => true,
    };
    if needs_build {
        let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
        let out = std::process::Command::new(&cc)
            .args(["-O2", "-shared", "-fPIC", "-o"])
            .arg(&so)
            .arg(&src)
            .arg("-ldl")
            .output();
        match out {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                eprintln!("shim build failed: {}", String::from_utf8_lossy(&o.stderr));
                return None;
            }
            Err(e) => {
                eprintln!("shim build error: {e}");
                return None;
            }
        }
    }
    Some(so)
}

pub fn wait_for_backend_healthy(ctx: &TestCtx, secs: u64) {
    for _ in 0..secs {
        if let Ok(r) = ctx.client.get(ctx.url("/get-questions-count/")).send() {
            if r.status().is_success() {
                return;
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    panic!("backend never became healthy at {}", ctx.base_url);
}

pub fn mailpit_message_count(ctx: &TestCtx) -> i64 {
    ctx.client
        .get(format!(
            "{}/api/v1/info",
            ctx.mailpit_url.trim_end_matches('/')
        ))
        .send()
        .ok()
        .and_then(|r| r.json::<serde_json::Value>().ok())
        .and_then(|v| {
            v.get("Messages")
                .and_then(|m| m.as_i64())
                .or_else(|| v.get("messages").and_then(|m| m.as_i64()))
        })
        .unwrap_or(0)
}

/// An authenticated session: a bare HTTP client plus the credentials needed
/// for the write endpoints.
pub struct AuthSession {
    pub client: reqwest::blocking::Client,
    /// Value for the `X-XSRF-Token` header.
    pub xsrf: String,
    /// Value for an explicit `Cookie` header.
    pub cookie: String,
    /// The authenticated user's id.
    pub user_id: i64,
}

impl AuthSession {
    /// Attach both auth headers to a request builder.
    pub fn auth(&self, rb: reqwest::blocking::RequestBuilder) -> reqwest::blocking::RequestBuilder {
        rb.header("Cookie", &self.cookie)
            .header("X-XSRF-Token", &self.xsrf)
    }
}

/// Register -> verify -> submit-email -> OTP login.
///
/// Returns an [`AuthSession`] rather than relying on a cookie jar, because
/// backend/src/users/routes.rs hard-codes `.secure(true)` on every cookie and
/// sets `Domain=<config.host>` (default `localhost:5173`). A `Secure` cookie is
/// never replayed over plain HTTP, and the domain does not match the test base
/// URL, so reqwest's jar silently drops them and every authenticated call
/// would fail with "Wrong XSRF token." Production runs behind TLS; here we
/// replay the `Set-Cookie` values verbatim, exactly as a browser over HTTPS
/// would.
///
/// Lives here (rather than in one test file) so both `api_writes_a` and
/// `api_writes_b` can use it without `#[path]` module aliasing, which would
/// otherwise create two distinct `TestCtx` types in one test binary.
pub fn login_capture(ctx: &TestCtx, prefix: &str) -> AuthSession {
    let client = reqwest::blocking::Client::builder()
        .build()
        .expect("reqwest client");
    let username = unique_username(prefix);
    let email = unique_email(prefix);
    let reg = client
        .post(ctx.url("/register"))
        .json(&serde_json::json!({"username": username, "email": email}))
        .send()
        .expect("register request");
    assert_eq!(reg.status().as_u16(), 200, "register {prefix}");
    let user_id = ctx.q_i64("SELECT id FROM users WHERE email = $1", &[&email]);
    ctx.exec(
        "UPDATE users SET email_verified = true WHERE email = $1",
        &[&email],
    );
    let sub = client
        .post(ctx.url("/submit-email"))
        .json(&serde_json::json!({"email": email}))
        .send()
        .expect("submit-email request");
    assert_eq!(sub.status().as_u16(), 200, "submit-email {prefix}");
    let otp = ctx.q_string(
        "SELECT token FROM tokens WHERE email = $1 ORDER BY created_date DESC LIMIT 1",
        &[&email],
    );
    let login = client
        .post(ctx.url("/auth/login"))
        .json(&serde_json::json!({"email": email, "otp": otp}))
        .send()
        .expect("login request");
    assert_eq!(login.status().as_u16(), 200, "login {prefix}");

    let mut pairs: Vec<String> = Vec::new();
    let mut xsrf: Option<String> = None;
    for v in login.headers().get_all(reqwest::header::SET_COOKIE).iter() {
        let raw = v.to_str().expect("set-cookie utf8");
        let pair = raw.split(';').next().unwrap_or("").trim().to_string();
        if pair.is_empty() {
            continue;
        }
        if let Some(v) = pair.strip_prefix("xsrf_token=") {
            xsrf = Some(v.to_string());
        }
        pairs.push(pair);
    }
    AuthSession {
        client,
        xsrf: xsrf.expect("xsrf_token cookie missing"),
        cookie: pairs.join("; "),
        user_id,
    }
}

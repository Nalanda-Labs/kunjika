use crate::state::*;
use crate::state::{redis::Client, KvPool, RedisConnectionManager};
use lettre::{
    transport::smtp::{
        authentication::Credentials,
        client::{Certificate, Tls, TlsParameters},
    },
    AsyncSmtpTransport, Tokio1Executor,
};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub sql: String,
    pub redis: String,
    /// Address:port the HTTP server binds to, e.g. `127.0.0.1:8000`.
    /// Under compose this is overridden to `0.0.0.0:8000` so the port
    /// mapping works; see `KUNJIKA_LISTEN` handling in `into_state`.
    pub listen: String,
    pub jwt_priv: String,
    pub mail_host: String,
    pub mail_port: u16,
    /// Upgrade the SMTP session to TLS via STARTTLS before sending mail.
    /// `false` keeps the old plaintext behaviour (e.g. a trusted local relay).
    #[serde(default)]
    pub mail_starttls: bool,
    /// Optional PEM file with extra CA/root certificates to trust for the
    /// STARTTLS handshake (e.g. a self-signed test cert). Relative paths
    /// resolve against the config file's directory; empty = system trust
    /// store only. Ignored when `mail_starttls` is false.
    #[serde(default)]
    pub mail_tls_ca: String,
    pub mail_username: String,
    pub mail_password: String,
    pub from_email: String,
    pub from_name: String,
    pub host: String,
    pub secret_key: String,
    pub email_verification_expiry_time: u64,
    pub questions_per_page: i32,
    pub users_per_page: i32,
    pub tags_per_page: i64,
    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_max_age: i64,
    pub refresh_token_private_key: String,
    pub refresh_token_public_key: String,
    pub refresh_token_max_age: i64,
    pub results_per_page: u8,
    pub karma_gain_per_vote: i64,
    pub karma_loss_per_vote: i64,
    pub upload_folder: String,
    pub backend_url: String,
    pub image_max_size: usize,
    pub karma_gain_per_answer: i64,
    pub login_code_length: usize,
}

impl Config {
    pub fn parse_from_file(file: &PathBuf) -> Self {
        use std::fs::read_to_string;

        info!("confp: {}", file.display());
        let confstr = read_to_string(file).expect("confile read");
        let mut cfg: Self = json5::from_str(&confstr).expect("confile read");
        // Compose/dev overrides: `podman-compose.yaml` sets these so the same
        // image works against container hostnames without editing the file.
        // Each is optional; unset means "keep the file value".
        if let Ok(v) = std::env::var("KUNJIKA_SQL_URL") {
            if !v.trim().is_empty() {
                cfg.sql = v;
            }
        }
        if let Ok(v) = std::env::var("KUNJIKA_REDIS_URL") {
            if !v.trim().is_empty() {
                cfg.redis = v;
            }
        }
        if let Ok(v) = std::env::var("KUNJIKA_LISTEN") {
            if !v.trim().is_empty() {
                cfg.listen = v;
            }
        }
        if let Ok(v) = std::env::var("KUNJIKA_MAIL_HOST") {
            if !v.trim().is_empty() {
                cfg.mail_host = v;
            }
        }
        if let Ok(v) = std::env::var("KUNJIKA_MAIL_PORT") {
            if let Ok(p) = v.trim().parse::<u16>() {
                cfg.mail_port = p;
            }
        }
        if let Ok(v) = std::env::var("KUNJIKA_HOST") {
            if !v.trim().is_empty() {
                cfg.host = v;
            }
        }
        if let Ok(v) = std::env::var("KUNJIKA_BACKEND_URL") {
            if !v.trim().is_empty() {
                cfg.backend_url = v;
            }
        }
        if let Ok(v) = std::env::var("KUNJIKA_UPLOAD_FOLDER") {
            if !v.trim().is_empty() {
                cfg.upload_folder = v;
            }
        }
        // A relative `mail_tls_ca` is resolved against the config file's own
        // directory so it doesn't depend on the process cwd.
        if !cfg.mail_tls_ca.is_empty() && !PathBuf::from(&cfg.mail_tls_ca).is_absolute() {
            if let Some(dir) = file.parent() {
                cfg.mail_tls_ca = dir.join(&cfg.mail_tls_ca).to_string_lossy().into_owned();
            }
        }
        cfg
    }
    pub async fn into_state(self) -> AppStateRaw {
        info!("config: {:?}", self);
        let pool_options = PoolOptions::new();

        let sql = pool_options.connect(&self.sql).await.expect("sql open");
        let kvm =
            RedisConnectionManager::new(Client::open(self.redis.clone()).expect("redis open"));
        let kv = KvPool::builder().build(kvm);
        match kv.get().await {
            Ok(_c) => info!("Connected to redis!"),
            Err(e) => panic!("Could not connect to Redis: {}", e),
        }
        match sql.try_acquire() {
            Some(_c) => info!("Connected to db!"),
            None => panic!("Could not connect to db!"),
        }

        // Mail transport. `mail_host`/`mail_port` may live on another
        // host/container (e.g. `mailpit:1025` under compose). When
        // `mail_starttls` is set, upgrade to TLS via STARTTLS (required, no
        // plaintext fallback); otherwise keep the plaintext behaviour.
        let mut mailer_builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            &self.mail_host,
        )
        .port(self.mail_port);
        if self.mail_starttls {
            // Optional extra CA/root certs (e.g. a self-signed test cert);
            // without them only the system trust store is used.
            let mut tls = TlsParameters::builder(self.mail_host.clone());
            if !self.mail_tls_ca.is_empty() {
                let pem = std::fs::read(&self.mail_tls_ca)
                    .unwrap_or_else(|e| panic!("mail_tls_ca read {}: {:?}", self.mail_tls_ca, e));
                let cert = Certificate::from_pem(&pem)
                    .unwrap_or_else(|e| panic!("mail_tls_ca parse {}: {:?}", self.mail_tls_ca, e));
                tls = tls.add_root_certificate(cert);
            }
            let tls = tls
                .build()
                .unwrap_or_else(|e| panic!("mail TLS parameters: {:?}", e));
            mailer_builder = mailer_builder.tls(Tls::Required(tls));
        }
        if !self.mail_username.is_empty() {
            mailer_builder = mailer_builder.credentials(Credentials::new(
                self.mail_username.clone(),
                self.mail_password.clone(),
            ));
        }
        let mailer = mailer_builder.build();
        match mailer.test_connection().await {
            Ok(_t) => info!("Test connection to mail server successful!"),
            Err(e) => panic!("Error connecting to mail server! {:?}", e),
        }

        Arc::new(State {
            config: self,
            sql,
            kv,
            mailer,
        })
    }
    // generate and show config string
    pub fn show() {
        let de: Self = Default::default();
        println!("{}", serde_json::to_string_pretty(&de).unwrap())
    }
}

// pub fn version_with_gitif() -> &'static str {
//     concat!(
//         env!("CARGO_PKG_VERSION"),
//         " ",
//         env!("VERGEN_GIT_COMMIT_DATE"),
//         ": ",
//         env!("VERGEN_GIT_SHA_SHORT")
//     )
// }

#[derive(clap::Parser, Debug)]
// #[clap(name = "template")]
// #[clap(version = version_with_gitif())]
pub struct Opts {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long)]
    pub verbose: u8,

    /// Config file
    #[clap(
        short = 'c',
        long = "config",
        // parse(from_os_str),
        default_value = "template.json"
    )]
    pub config: PathBuf,
}

impl Opts {
    pub fn parse_from_args() -> (JoinHandle, Self) {
        use clap::Parser;
        let opt: Self = Opts::parse();

        let level = match opt.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _more => LevelFilter::Trace,
        };

        let formatter = BaseFormater::new()
            .local(true)
            .color(true)
            .level(4)
            .formater(format);
        let filter = BaseFilter::new()
            .starts_with(true)
            .notfound(true)
            .max_level(level)
            .chain(
                "sqlx",
                if opt.verbose > 1 {
                    LevelFilter::Debug
                } else {
                    LevelFilter::Warn
                },
            );
        info!("Some message");

        let handle = NonblockLogger::new()
            .filter(filter)
            .unwrap()
            .formater(formatter)
            .log_to_stdout()
            .map_err(|e| eprintln!("failed to init nonblock_logger: {:?}", e))
            .unwrap();

        info!("opt: {:?}", opt);

        (handle, opt)
    }
}

use nonblock_logger::{
    log::{LevelFilter, Record},
    BaseFilter, BaseFormater, FixedLevel, JoinHandle, NonblockLogger,
};

pub fn format(base: &BaseFormater, record: &Record) -> String {
    let level = FixedLevel::with_color(record.level(), base.color_get())
        .length(base.level_get())
        .into_colored()
        .into_coloredfg();

    format!(
        "[{} {}#{}:{} {}] {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
        level,
        record.module_path().unwrap_or("*"),
        record.file().unwrap_or("*"),
        record.line().unwrap_or(0),
        // nonblock_logger::current_thread_name(),
        record.args()
    )
}

use crate::state::*;
use crate::state::{redis::Client, KvPool, RedisConnectionManager};
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub sql: String,
    pub redis: String,
    pub listen: String,
    pub jwt_priv: String,
    pub mail_host: String,
    pub mail_port: u16,
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
        json5::from_str(&confstr).expect("confile deser")
    }
    pub async fn into_state(self) -> AppStateRaw {
        info!("config: {:?}", self);
        let pool_options = PoolOptions::new();

        let sql = pool_options.connect(&self.sql).await.expect("sql open");
        let kvm =
            RedisConnectionManager::new(Client::open(self.redis.clone()).expect("redis open"));
        let kv = KvPool::builder().build(kvm);
        let smtp_credentials =
            Credentials::new(self.mail_username.clone(), self.mail_password.clone());

        // let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.mail_host)
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost();
        // .unwrap()
        // .credentials(smtp_credentials)
        // .build();

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

        let formater = BaseFormater::new()
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

        let handle = NonblockLogger::new()
            .filter(filter)
            .unwrap()
            .formater(formater)
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

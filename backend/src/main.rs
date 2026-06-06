#![recursion_limit = "256"]
#[macro_use]
extern crate nonblock_logger;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate validator;
#[macro_use]
extern crate sqlx;
#[macro_use]
extern crate serde;

use nix::unistd::Uid;
use ntex::{web, web::App, web::HttpServer};
use ntex_cors::Cors;
use num_cpus;

// pub mod api;
pub mod config;
// pub mod how;
pub mod middlewares;
// pub mod models;
pub mod questions;
pub mod state;
pub mod tags;
pub mod uploads;
pub mod users;
pub mod utils;
pub mod votes;
use tokio::sync::OnceCell;

use config::{Config, Opts};
use std::sync::Arc;
use state::State;


const APIV1:&'static str = "/api/v1";
const LISTEN: &'static str = "127.0.0.1:800";

#[ntex::main]
async fn main() -> std::io::Result<()> {
    // we do not allow the app to run as root simply because
    // it is dangerous
    if Uid::effective().is_root() {
        eprintln!("Error: Do not run     this program as root.");
        std::process::exit(1);
    }
    // Config::show();
    let (_handle, opt) = Opts::parse_from_args();
    static STATE: OnceCell<Arc<State>> = OnceCell::const_new();
    let state = STATE
    .get_or_init(|| async {
        Config::parse_from_file(&opt.config)
            .into_state()
            .await
    })
    .await;

    HttpServer::new(async || {
        App::new()
            .middleware(
                Cors::new()
                    // we can easily place these in a config file but I find
                    // it unnecessary as these settings are default for what
                    // devs have and it has no bearing on anything else
                    // for svelte dev server
                    .allowed_origin("http://localhost:5173")
                    // for node production server
                    .allowed_origin("http://localhost:3000")
                    // when using nginx as a reverse proxy to just use localhost
                    .allowed_origin("http://localhost")
                    .supports_credentials()
                    .max_age(3600)
                    .finish(),
            )
            .state(state.clone())
            .middleware(web::middleware::Logger::default())
            .middleware(web::middleware::Compress::default())
            .service((web::scope(APIV1)
                .configure(users::routes::init)
                .configure(tags::routes::init)
                .configure(votes::routes::init)
                .configure(questions::routes::init)
                .configure(uploads::routes::init),))
    })
    .workers(num_cpus::get())   
    .bind(LISTEN)?
    .run()
    .await
}

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

use ntex::{web, web::App, web::HttpServer};
use ntex_cors::Cors;
use ntex_files as fs;
use num_cpus;

// pub mod api;
pub mod config;
// pub mod how;
pub mod middlewares;
// pub mod models;
pub mod questions;
pub mod state;
pub mod tags;
pub mod users;
pub mod utils;
pub mod votes;

use config::{Config, Opts};

#[ntex::main]
async fn main() -> std::io::Result<()> {
    // Config::show();
    let (_handle, opt) = Opts::parse_from_args();
    let state = Config::parse_from_file(&opt.config).into_state().await;
    let state2 = state.clone();
    let apiv1 = "/api/v1";

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new()
                    // for svelte dev server
                    .allowed_origin("http://localhost:5173")
                    // for node production server
                    .allowed_origin("http://localhost:3000")
                    .supports_credentials()
                    .max_age(3600)
                    .finish(),
            )
            .state(state.clone())
            .wrap(web::middleware::Logger::default())
            .wrap(web::middleware::Compress::default())
            .service(
                (web::scope(apiv1)
                    .configure(users::routes::init)
                    .configure(tags::routes::init)
                    .configure(votes::routes::init)
                    .configure(questions::routes::init),
            ))
    })
    .workers(num_cpus::get())
    .keep_alive(300)
    .bind(&state2.config.listen)?
    .run()
    .await
}

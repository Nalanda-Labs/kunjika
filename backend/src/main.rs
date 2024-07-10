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

use actix_web::{web, App, HttpServer, middleware};
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Config::show();
    let (_handle, opt) = Opts::parse_from_args();
    let state = Config::parse_from_file(&opt.config).into_state().await;
    let state2 = state.clone();
    let apiv1 = "/api/v1";

    HttpServer::new(move || {
        App::new()
            // we do not do CORS rather use nginx to reverse proxy everything to localhost
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(
                web::scope(apiv1)
                    .configure(users::routes::init)
                    .configure(tags::routes::init)
                    .configure(votes::routes::init)
                    .configure(questions::routes::init),
            )
    })
    .workers(num_cpus::get())
    .keep_alive(std::time::Duration::from_secs(300))
    .bind(&state2.config.listen)?
    .run()
    .await
}

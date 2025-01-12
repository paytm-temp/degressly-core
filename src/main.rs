mod proxy;
mod kafka;
mod config;
mod models;

use actix_web::{web, App, HttpServer};
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting degressly-core server...");

    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

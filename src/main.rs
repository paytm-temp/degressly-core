mod proxy;
mod kafka;
mod config;
mod models;

use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use log::info;
use proxy::{
    handler::ProxyHandler,
    service::HttpProxyMulticastService,
};
use kafka::{
    config::KafkaConfig,
    producer::ProducerTemplate,
    replay::ReplayReceiver,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting degressly-core server...");

    // TODO: Move to configuration
    // Load configuration
    let settings = config::Settings::new()
        .expect("Failed to load configuration");

    // Initialize Kafka configuration
    let kafka_config = kafka::config::KafkaConfig::new(
        settings.kafka.bootstrap_servers,
        settings.kafka.group_id,
    );

    // Create Kafka producer and consumer
    let producer = kafka_config.create_producer();
    let producer_template = Arc::new(ProducerTemplate::new(producer));

    // Create multicast service
    let multicast_service = Arc::new(HttpProxyMulticastService::new(
        settings.hosts.primary,
        settings.hosts.secondary,
        settings.hosts.candidate,
    ));

    // Initialize replay receiver with consumer
    let replay_consumer = kafka_config.create_consumer(&[&settings.kafka.replay_topic]);
    let replay_receiver = ReplayReceiver::new(replay_consumer, Arc::clone(&multicast_service));

    // Start replay loop in background task
    let replay_handle = tokio::spawn(async move {
        if let Err(e) = replay_receiver.start_replay_loop().await {
            log::error!("Replay loop error: {}", e);
        }
    });

    let proxy_handler = web::Data::new(ProxyHandler::new(multicast_service));

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(proxy_handler.clone())
            .service(web::scope("/api").service(
                web::resource("/proxy").route(web::post().to(ProxyHandler::handle_proxy))
            ))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

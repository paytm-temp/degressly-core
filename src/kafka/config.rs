use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;
use rdkafka::consumer::{StreamConsumer, Consumer};
use std::time::Duration;

pub struct KafkaConfig {
    bootstrap_servers: String,
    group_id: String,
}

impl KafkaConfig {
    pub fn new(bootstrap_servers: String, group_id: String) -> Self {
        Self {
            bootstrap_servers,
            group_id,
        }
    }

    pub fn create_producer(&self) -> FutureProducer {
        ClientConfig::new()
            .set("bootstrap.servers", &self.bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation failed")
    }

    pub fn create_consumer(&self, topics: &[&str]) -> StreamConsumer {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.bootstrap_servers)
            .set("group.id", &self.group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "6000")
            .create()
            .expect("Consumer creation failed");

        consumer
            .subscribe(topics)
            .expect("Topic subscription failed");

        consumer
    }
}

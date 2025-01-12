use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use serde::Serialize;
use crate::models::Result;

pub struct ProducerTemplate {
    producer: FutureProducer,
}

impl ProducerTemplate {
    pub fn new(producer: FutureProducer) -> Self {
        Self { producer }
    }

    pub async fn send_message<T: Serialize>(
        &self,
        topic: &str,
        key: Option<&str>,
        message: &T,
    ) -> Result<()> {
        let payload = serde_json::to_string(message)
            .map_err(|e| crate::models::DegresslyError::KafkaError(e.to_string()))?;

        let record = FutureRecord::to(topic)
            .payload(&payload)
            .key(key.unwrap_or(""));

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| crate::models::DegresslyError::KafkaError(e.to_string()))?;

        Ok(())
    }
}

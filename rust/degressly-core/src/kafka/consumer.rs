use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::message::Message;
use rdkafka::message::BorrowedMessage;
use serde::de::DeserializeOwned;
use crate::models::Result;

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(consumer: StreamConsumer) -> Self {
        Self { consumer }
    }

    pub async fn consume_message<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        match self.consumer.recv().await {
            Ok(message) => {
                let payload = message
                    .payload()
                    .ok_or_else(|| crate::models::DegresslyError::KafkaError("Empty message".into()))?;

                let parsed: T = serde_json::from_slice(payload)
                    .map_err(|e| crate::models::DegresslyError::KafkaError(e.to_string()))?;

                Ok(Some(parsed))
            }
            Err(e) => Err(crate::models::DegresslyError::KafkaError(e.to_string())),
        }
    }
}

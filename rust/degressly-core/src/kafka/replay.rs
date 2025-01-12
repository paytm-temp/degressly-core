use std::sync::Arc;
use rdkafka::consumer::StreamConsumer;
use crate::models::{DegresslyRequest, Result};
use crate::proxy::MulticastService;
use super::consumer::KafkaConsumer;

pub struct ReplayReceiver {
    consumer: KafkaConsumer,
    multicast_service: Arc<dyn MulticastService>,
}

impl ReplayReceiver {
    pub fn new(consumer: StreamConsumer, multicast_service: Arc<dyn MulticastService>) -> Self {
        Self {
            consumer: KafkaConsumer::new(consumer),
            multicast_service,
        }
    }

    pub async fn start_replay_loop(&self) -> Result<()> {
        loop {
            if let Some(request) = self.consumer.consume_message::<DegresslyRequest>().await? {
                // Always wait for all replicas in replay mode
                self.multicast_service.get_response(request, true).await?;
            }
        }
    }
}

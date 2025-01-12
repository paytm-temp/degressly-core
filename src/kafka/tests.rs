#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use rdkafka::producer::FutureProducer;
    use rdkafka::consumer::StreamConsumer;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        KafkaConfig {}
        impl KafkaConfig {
            fn create_producer(&self) -> FutureProducer;
            fn create_consumer(&self, topics: &[&str]) -> StreamConsumer;
        }
    }

    #[tokio::test]
    async fn test_producer_template_send_message() {
        // Arrange
        let producer = FutureProducer::from_config(&rdkafka::ClientConfig::new())
            .expect("Producer creation failed");
        let template = ProducerTemplate::new(producer);
        let message = DegresslyRequest {
            trace_id: "test-trace".to_string(),
            method: "POST".to_string(),
            url: "/test".to_string(),
            headers: HashMap::new(),
            body: Some(vec![]),
            params: HashMap::new(),
        };

        // Act & Assert
        let result = template
            .send_message("test.topic", Some("test-key"), &message)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_replay_receiver() {
        // Arrange
        let mut mock_service = MockMulticastService::new();
        mock_service
            .expect_get_response()
            .times(1)
            .returning(|_, _| Ok(HashMap::new()));

        let consumer = StreamConsumer::from_config(&rdkafka::ClientConfig::new())
            .expect("Consumer creation failed");
        let receiver = ReplayReceiver::new(consumer, Arc::new(mock_service));

        // Note: In a real test environment, we would:
        // 1. Set up test Kafka broker
        // 2. Produce test messages
        // 3. Verify replay behavior
        // For now, we just verify the receiver can be created
        assert!(receiver.start_replay_loop().await.is_ok());
    }
}

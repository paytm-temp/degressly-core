#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use std::sync::Arc;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        MulticastService {}
        #[async_trait]
        impl MulticastService for MulticastService {
            async fn get_response(
                &self,
                request: DegresslyRequest,
                wait_for_all_replicas: bool,
            ) -> Result<HashMap<HostType, DownstreamResult>>;
        }
    }

    #[actix_web::test]
    async fn test_handle_proxy_basic_routing() {
        // Arrange
        let mut mock_service = MockMulticastService::new();
        mock_service
            .expect_get_response()
            .times(1)
            .returning(|_, _| {
                Ok(HashMap::from([(
                    HostType::Primary,
                    DownstreamResult {
                        headers: HashMap::new(),
                        status_code: 200,
                        body: Some(b"test response".to_vec()),
                        error: None,
                    },
                )]))
            });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_service) as Arc<dyn MulticastService>))
                .service(web::resource("/proxy").route(web::post().to(handle_proxy))),
        )
        .await;

        // Act
        let req = test::TestRequest::post()
            .uri("/proxy")
            .set_payload("test body")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_handle_proxy_wait_for_all() {
        // Arrange
        let mut mock_service = MockMulticastService::new();
        mock_service
            .expect_get_response()
            .with(predicate::always(), predicate::eq(true))
            .times(1)
            .returning(|_, _| {
                Ok(HashMap::from([
                    (
                        HostType::Primary,
                        DownstreamResult {
                            headers: HashMap::new(),
                            status_code: 200,
                            body: Some(b"primary response".to_vec()),
                            error: None,
                        },
                    ),
                    (
                        HostType::Secondary,
                        DownstreamResult {
                            headers: HashMap::new(),
                            status_code: 200,
                            body: Some(b"secondary response".to_vec()),
                            error: None,
                        },
                    ),
                ]))
            });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(mock_service) as Arc<dyn MulticastService>))
                .service(web::resource("/proxy").route(web::post().to(handle_proxy))),
        )
        .await;

        // Act
        let req = test::TestRequest::post()
            .uri("/proxy")
            .insert_header(("X-Wait-For-All", "true"))
            .set_payload("test body")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), 200);
    }
}

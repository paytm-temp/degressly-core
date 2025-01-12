use actix_web::{
    post,
    web::{self, Bytes},
    HttpRequest, HttpResponse,
};
use std::sync::Arc;

use crate::{
    models::{DegresslyRequest, Result},
    proxy::MulticastService,
};

pub struct ProxyHandler {
    multicast_service: Arc<dyn MulticastService>,
}

impl ProxyHandler {
    pub fn new(multicast_service: Arc<dyn MulticastService>) -> Self {
        Self { multicast_service }
    }

    #[post("/proxy")]
    pub async fn handle_proxy(
        &self,
        req: HttpRequest,
        body: Bytes,
        query: web::Query<HashMap<String, String>>,
    ) -> Result<HttpResponse> {
        let degressly_request = DegresslyRequest {
            trace_id: req
                .headers()
                .get("X-Trace-ID")
                .and_then(|h| h.to_str().ok())
                .unwrap_or_default()
                .to_string(),
            method: req.method().to_string(),
            url: req.uri().to_string(),
            headers: req
                .headers()
                .iter()
                .map(|(name, value)| {
                    (
                        name.to_string(),
                        vec![value.to_str().unwrap_or_default().to_string()],
                    )
                })
                .collect(),
            body: Some(body.to_vec()),
            params: query
                .into_inner()
                .into_iter()
                .map(|(k, v)| (k, vec![v]))
                .collect(),
        };

        let wait_for_all = req
            .headers()
            .get("X-Wait-For-All")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let results = self.multicast_service.get_response(degressly_request, wait_for_all).await?;
        
        // For now, return the primary result if available, or the first available result
        let primary_result = results
            .get(&HostType::Primary)
            .or_else(|| results.values().next())
            .ok_or_else(|| crate::models::DegresslyError::InternalError("No response available".into()))?;

        Ok(HttpResponse::build(
            actix_web::http::StatusCode::from_u16(primary_result.status_code)
                .unwrap_or(actix_web::http::StatusCode::OK),
        )
        .body(primary_result.body.clone().unwrap_or_default()))
    }
}

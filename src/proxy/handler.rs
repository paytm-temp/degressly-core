use actix_web::{
    web::{self, Bytes},
    HttpRequest, HttpResponse,
};
use std::{collections::HashMap, sync::Arc};

use crate::{
    models::{DegresslyRequest, HostType, DegresslyError},
    proxy::MulticastService,
};

pub async fn handle_proxy(
    multicast_service: web::Data<Arc<dyn MulticastService>>,
    req: HttpRequest,
    body: Bytes,
    query: web::Query<HashMap<String, String>>,
) -> actix_web::Result<HttpResponse> {
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

    let results = multicast_service
        .get_response(degressly_request, wait_for_all)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    // For now, return the primary result if available, or the first available result
    let primary_result = results
        .get(&HostType::Primary)
        .or_else(|| results.values().next())
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("No response available"))?;

    // Build response with headers
    let mut response = HttpResponse::build(
        actix_web::http::StatusCode::from_u16(primary_result.status_code)
            .unwrap_or(actix_web::http::StatusCode::OK)
    );

    // Add headers from primary result
    for (name, values) in &primary_result.headers {
        for value in values {
            response.append_header((name.clone(), value.clone()));
        }
    }

    Ok(response.body(primary_result.body.clone().unwrap_or_default()))
}

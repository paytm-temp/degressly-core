use std::collections::HashMap;
// Removed unused Arc import

use async_trait::async_trait;
use reqwest::Client;

use crate::models::{DegresslyRequest, DegresslyError, DownstreamResult, HostType, Result};
use super::MulticastService;

pub struct HttpProxyMulticastService {
    client: Client,
    primary_host: String,
    secondary_host: String,
    candidate_host: String,
}

impl HttpProxyMulticastService {
    pub fn new(primary_host: String, secondary_host: String, candidate_host: String) -> Self {
        Self {
            client: Client::new(),
            primary_host,
            secondary_host,
            candidate_host,
        }
    }

    async fn make_request_static(client: &Client, base_url: &str, request: DegresslyRequest) -> Result<DownstreamResult> {
        let url = format!("{}{}", &base_url, request.url);
        let mut req_builder = client
            .request(
                reqwest::Method::from_bytes(request.method.as_bytes())
                    .map_err(|e| DegresslyError::HttpError(e.to_string()))?,
                &url,
            );

        // Add headers
        for (name, values) in &request.headers {
            for value in values {
                req_builder = req_builder.header(name, value);
            }
        }

        // Add body if present
        if let Some(body) = &request.body {
            req_builder = req_builder.body(body.clone());
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| DegresslyError::HttpError(e.to_string()))?;

        let status = response.status().as_u16();
        let headers: HashMap<String, Vec<String>> = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    vec![value.to_str().unwrap_or_default().to_string()],
                )
            })
            .collect();

        let body = response
            .bytes()
            .await
            .map_err(|e| DegresslyError::HttpError(e.to_string()))?
            .to_vec();

        Ok(DownstreamResult {
            headers,
            status_code: status,
            body: Some(body),
            error: None,
        })
    }
}

#[async_trait]
impl MulticastService for HttpProxyMulticastService {
    async fn get_response(
        &self,
        request: DegresslyRequest,
        wait_for_all_replicas: bool,
    ) -> Result<HashMap<HostType, DownstreamResult>> {
        let mut results = HashMap::new();
        
        // Clone everything needed for async blocks
        let client = self.client.clone();
        let primary_host = self.primary_host.clone();
        let secondary_host = self.secondary_host.clone();
        let candidate_host = self.candidate_host.clone();
        
        // Create independent futures that own their data
        let primary_fut = {
            let req = request.clone();
            let host = primary_host;
            async move {
                HttpProxyMulticastService::make_request_static(&client, &host, req).await
            }
        };
        
        let secondary_fut = {
            let req = request.clone();
            let host = secondary_host;
            async move {
                HttpProxyMulticastService::make_request_static(&client, &host, req).await
            }
        };
        
        let candidate_fut = {
            let req = request;
            let host = candidate_host;
            async move {
                HttpProxyMulticastService::make_request_static(&client, &host, req).await
            }
        };

        if wait_for_all_replicas {
            // Wait for all requests to complete
            let (primary_result, secondary_result, candidate_result) = 
                tokio::try_join!(primary_fut, secondary_fut, candidate_fut)
                    .map_err(|e| DegresslyError::HttpError(e.to_string()))?;

            results.insert(HostType::Primary, primary_result);
            results.insert(HostType::Secondary, secondary_result);
            results.insert(HostType::Candidate, candidate_result);
        } else {
            // Only wait for primary, fire and forget others
            if let Ok(result) = primary_fut.await {
                results.insert(HostType::Primary, result);
            }
            
            tokio::spawn(async move {
                if let Ok(_result) = secondary_fut.await {
                    // TODO: Implement observation publishing for secondary result
                }
            });
            
            tokio::spawn(async move {
                if let Ok(_result) = candidate_fut.await {
                    // TODO: Implement observation publishing for candidate result
                }
            });
        }

        Ok(results)
    }
}

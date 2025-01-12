use crate::models::{DegresslyRequest, DownstreamResult, HostType, Result};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait MulticastService: Send + Sync {
    async fn get_response(
        &self,
        request: DegresslyRequest,
        wait_for_all_replicas: bool,
    ) -> Result<HashMap<HostType, DownstreamResult>>;
}

pub mod handler;
pub mod service;

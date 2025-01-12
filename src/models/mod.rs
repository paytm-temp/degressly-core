use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegresslyRequest {
    pub trace_id: String,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Option<Vec<u8>>,
    pub params: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownstreamResult {
    pub headers: HashMap<String, Vec<String>>,
    pub status_code: u16,
    pub body: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub trace_id: String,
    pub request: DegresslyRequest,
    pub downstream_results: HashMap<String, DownstreamResult>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePopulationRequest {
    pub method: String,
    pub url: String,
    pub status_code: u16,
    pub body: Option<Vec<u8>>,
    pub headers: HashMap<String, Vec<String>>,
}

// Enum to represent different host types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostType {
    Primary,
    Secondary,
    Candidate,
}

// Error types for the application
#[derive(Debug, thiserror::Error)]
pub enum DegresslyError {
    #[error("HTTP error: {0}")]
    HttpError(String),
    
    #[error("Kafka error: {0}")]
    KafkaError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl actix_web::ResponseError for DegresslyError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": self.to_string()
            }))
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            DegresslyError::HttpError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type Result<T> = std::result::Result<T, DegresslyError>;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub hosts: HostsConfig,
    pub kafka: KafkaConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub debug: bool,
}

#[derive(Debug, Deserialize)]
pub struct HostsConfig {
    pub primary: String,
    pub secondary: String,
    pub candidate: String,
}

#[derive(Debug, Deserialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub group_id: String,
    pub replay_topic: String,
    pub observation_topic: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start with default settings
            .add_source(File::with_name("config/default.yaml"))
            // Add environment-specific config
            .add_source(File::with_name(&format!("config/{}.yaml", run_mode)).required(false))
            // Add environment variables with prefix "APP_"
            .add_source(Environment::with_prefix("APP"))
            .build()?;

        s.try_deserialize()
    }
}

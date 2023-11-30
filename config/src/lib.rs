use axum_on_rails::config::{DatabaseConfig, ServerConfig};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    // add your config settings here…
}

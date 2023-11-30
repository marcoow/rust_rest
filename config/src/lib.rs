use axum_on_rails::config::ServerConfig;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    // add your config settings here…
}

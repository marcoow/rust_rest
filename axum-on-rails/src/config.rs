use serde::Deserialize;
use std::{net::SocketAddr, str::FromStr};

#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub interface: String,
    pub port: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            // TODO: this should be 0.0.0.0 for production
            interface: String::from("127.0.0.1"),
            port: 3000,
        }
    }
}

impl ServerConfig {
    pub fn get_bind_addr(&self) -> SocketAddr {
        SocketAddr::from_str(format!("{}:{}", self.interface, self.port).as_str()).unwrap_or_else(
            |_| {
                panic!(
                    r#"Could not parse bind addr "{}:{}"!"#,
                    self.interface, self.port
                )
            },
        )
    }
}

use figment::{
    providers::{Format, Toml, Env},
    Figment,
};
use serde::de::Deserialize;
use std::fmt::{Display, Formatter, Result};
use std::env;
use tracing::info;
use tracing_panic::panic_hook;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
            Environment::Test => write!(f, "test"),
        }
    }
}

pub fn get_env() -> Environment {
    // TODO: come up with a better name for the env var!
    match env::var("APP_ENVIRONMENT") {
        Ok(val) => {
            let env = match val.to_lowercase().as_str() {
                "dev" | "development" => Environment::Development,
                "prod" | "production" => Environment::Production,
                "test" => Environment::Test,
                unknown => {
                    panic!(r#"Unknown environment: "{}"!"#, unknown);
                }
            };
            info!("Setting environment from APP_ENVIRONMENT: {}", env);
            env
        }
        Err(_) => {
            info!("Defaulting to environment: development");
            Environment::Development
        }
    }
}

pub fn load_config<'a, T>() -> T
where
    T: Deserialize<'a>,
{
    let environment = get_env();
    let env_config_file = match environment {
        Environment::Development => "development.toml",
        Environment::Production => "production.toml",
        Environment::Test => "test.toml",
    };

    let config: T = Figment::new()
        .merge(Toml::file("config/app.toml"))
        .merge(Toml::file(format!(
            "config/environments/{}",
            env_config_file
        )))
        .merge(Env::prefixed("SERVER_").map(|k| format!("server.{}", k.as_str()).into()))
        .extract()
        .expect("Could not read configuration!");
    config
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    std::panic::set_hook(Box::new(panic_hook));
}

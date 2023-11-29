use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::de::Deserialize;
use std::{env, net::SocketAddr, str::FromStr};
use tracing_panic::panic_hook;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub enum Environment {
    Development,
    Production,
    Test,
}

pub fn get_env() -> Environment {
    // TODO: come up with a better name for the env var!
    match env::var("APP_ENVIRONMENT") {
        Ok(val) => match val.to_lowercase().as_str() {
            "dev" | "development" => Environment::Development,
            "prod" | "production" => Environment::Production,
            "test" => Environment::Test,
            unknown => {
                panic!(r#"Unknown environment: "{}"!"#, unknown);
            }
        },
        Err(_) => Environment::Development,
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
        .extract()
        .expect("Could not read configuration!");
    config
}

pub fn get_bind_addr() -> SocketAddr {
    // TODO: come up with a better name for the env var!
    let iface = match env::var("APP_BIND_IFACE") {
        Ok(val) => val,
        Err(_) => String::from("127.0.0.1"),
    };
    let port = match env::var("APP_PORT") {
        Ok(val) => val,
        Err(_) => String::from("3000"),
    };

    SocketAddr::from_str(format!("{}:{}", iface, port).as_str())
        .unwrap_or_else(|_| panic!(r#"Could not parse bind addr "{}:{}"!"#, iface, port))
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

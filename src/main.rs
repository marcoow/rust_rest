use axum::http::StatusCode;
use axum_on_rails::load_config;
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tracing::{debug, Level};
use tracing_panic::panic_hook;
use tracing_subscriber::FmtSubscriber;

mod config;
mod controllers;
mod entities;
mod middlewares;
mod routes;
mod state;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let log_level = get_log_level();
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    std::panic::set_hook(Box::new(panic_hook));

    let config = load_config();

    let app_state = state::app_state(config).await;
    let app = routes::routes(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn get_log_level() -> Level {
    match env::var("RUST_LOG") {
        Ok(val) => match val.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            unknown => {
                eprintln!(r#"Unknown log level: "{}"!"#, unknown);
                std::process::exit(1)
            }
        },
        Err(_) => Level::INFO,
    }
}

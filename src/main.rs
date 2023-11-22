use axum::http::StatusCode;
use axum_on_rails::{get_bind_addr, get_log_level, load_config};
use dotenvy::dotenv;
use tracing::info;
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

    let addr = get_bind_addr();
    info!("Listening on {}", addr);
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

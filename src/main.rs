use axum::http::StatusCode;
use std::net::SocketAddr;
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;

mod controllers;
mod middlewares;
mod routes;
mod state;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let app_state = state::app_state().await;
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

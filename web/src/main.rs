use axum::http::StatusCode;
use axum_on_rails::{get_bind_addr, init_tracing, load_config};
use dotenvy::dotenv;
use tracing::info;

mod controllers;
mod middlewares;
mod routes;
mod state;

#[cfg(test)]
mod test;

pub async fn run() -> anyhow::Result<()> {
    dotenv().ok();

    let config: rust_rest_config::Config = load_config();

    let app_state = state::app_state(config).await;
    let app = routes::routes(app_state);

    let addr = get_bind_addr();
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(e) = run().await {
        tracing::error!(
            error.msg = %e,
            error.error_chain = ?e,
            "Shutting down due to error"
        )
    }
}

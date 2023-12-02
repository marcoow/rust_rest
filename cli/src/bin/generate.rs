use axum_on_rails::cli::generate::cli;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    cli().await;
}

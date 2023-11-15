use axum_on_rails::cli::db::cli;

#[tokio::main]
async fn main() {
    cli().await;
}

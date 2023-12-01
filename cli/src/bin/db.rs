use axum_on_rails::{load_config, cli::db::cli};
use crate::config::Config;

#[tokio::main]
async fn main() {
    let config: Config = load_config();
    cli(&config).await;
}

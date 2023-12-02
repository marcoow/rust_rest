use axum_on_rails::{cli::db::cli, load_config_for_env};
use rust_rest_config::Config;

#[tokio::main]
async fn main() {
    cli(|env| {
        let config: Config = load_config_for_env(&env);
        config.database
    })
    .await;
}

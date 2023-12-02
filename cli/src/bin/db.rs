use axum_on_rails::{cli::db::cli, load_config, Environment};
use dotenvy::dotenv;
use rust_rest_config::Config;

#[tokio::main]
async fn main() {
    cli(|env| {
        match env {
            Environment::Development => {
                dotenv().ok();
            }
            Environment::Test => {
                dotenvy::from_filename(".env.test").ok();
            }
            _ => { /* don't use and .env file for production */ }
        }
        dotenv().ok();

        let config: Config = load_config();
        config.database
    })
    .await;
}

use pacesetter::{cli::db::cli, load_config};
use rust_rest_config::Config;

#[tokio::main]
async fn main() {
    cli(|env| {
        let config: Config = load_config(&env);
        config.database
    })
    .await;
}

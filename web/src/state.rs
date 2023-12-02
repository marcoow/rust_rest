use rust_rest_config::Config;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: Config,
}

pub async fn app_state(config: Config) -> AppState {
    let db_pool = PgPoolOptions::new()
        .connect(config.database.url.as_str())
        .await
        .expect("Could not connect to database!");

    AppState { db_pool, config }
}

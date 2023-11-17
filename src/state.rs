use dotenvy::dotenv;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

pub async fn app_state() -> AppState {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("No DATABASE_URL set – cannot start server!");

    let db_pool = PgPoolOptions::new()
        .connect(db_url.as_str())
        .await
        .expect("Could not connect to database!");

    AppState { db_pool }
}

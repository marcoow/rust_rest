use crate::routes::routes;
use crate::state::AppState;
use axum_on_rails::{
    load_config,
    test::helpers::{build_test_context, prepare_db, TestContext},
};
use rust_rest_config::Config;
use sqlx::postgres::PgPoolOptions;

pub async fn setup() -> TestContext {
    dotenvy::from_filename(".env.test").ok();

    let config: Config = load_config();

    let db_config = prepare_db(&config.database).await;
    let db_pool = PgPoolOptions::new()
        .connect_with(db_config.clone())
        .await
        .expect("Could not connect to database!");

    let app = routes(AppState {
        db_pool: db_pool.clone(),
        config,
    });

    build_test_context(app, db_pool, db_config)
}

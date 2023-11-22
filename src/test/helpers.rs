use crate::routes::routes;
use crate::state::AppState;
use axum_on_rails::{
    load_config,
    test::helpers::{build_test_context, prepare_db, TestContext},
};
use sqlx::postgres::PgPoolOptions;

pub async fn setup() -> TestContext {
    let config = load_config();

    let db_config = prepare_db().await;
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

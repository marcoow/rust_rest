use axum_on_rails::{
    load_config_for_env,
    test::helpers::{build_test_context, prepare_db, TestContext},
    Environment,
};
use rust_rest_config::Config;
use rust_rest_web::routes::routes;
use rust_rest_web::state::AppState;
use sqlx::postgres::PgPoolOptions;

static CONFIG: tokio::sync::OnceCell<Config> = tokio::sync::OnceCell::const_new();

pub async fn setup() -> TestContext {
    let config: &Config = CONFIG
        .get_or_init(|| async { load_config_for_env::<Config>(&Environment::Test) })
        .await;

    let db_config = prepare_db(&config.database).await;
    let db_pool = PgPoolOptions::new()
        .connect_with(db_config.clone())
        .await
        .expect("Could not connect to database!");

    let app = routes(AppState {
        env: Environment::Test,
        db_pool: db_pool.clone(),
        config: config.clone(),
    });

    build_test_context(app, db_pool, db_config)
}

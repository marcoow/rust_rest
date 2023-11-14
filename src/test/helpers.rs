use crate::routes::routes;
use crate::state::AppState;
use axum_on_rails::test_helpers::{prepare_db, TestSetup, build_test_context};
use tokio_postgres::NoTls;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

pub async fn setup() -> TestSetup {
    let test_db_config = prepare_db().await;
    let manager = PostgresConnectionManager::new(test_db_config.clone(), NoTls);
    let pool = Pool::builder().build(manager).await.unwrap();

    let app = routes(AppState {
        db_pool: pool.clone(),
    });

    build_test_context(app, pool, test_db_config)
}

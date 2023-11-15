use crate::routes::routes;
use crate::state::AppState;
use axum_on_rails::test::helpers::{build_test_context, prepare_db, TestContext};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub async fn setup() -> TestContext {
    let db_config = prepare_db().await;
    let manager = PostgresConnectionManager::new(db_config.clone(), NoTls);
    let pool = Pool::builder().build(manager).await.unwrap();

    let app = routes(AppState {
        db_pool: pool.clone(),
    });

    build_test_context(app, pool, db_config)
}

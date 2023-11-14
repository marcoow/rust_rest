use axum_on_rails::ConnectionPool;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenvy::dotenv;
use std::env;
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: ConnectionPool,
}

pub async fn app_state() -> AppState {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("No DATABASE_URL set – cannot start server!");

    let manager = PostgresConnectionManager::new_from_stringlike(&db_url, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    AppState { db_pool: pool }
}

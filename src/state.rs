use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenvy_macro::dotenv;
use tokio_postgres::NoTls;

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: ConnectionPool,
}

pub async fn app_state() -> AppState {
    let db_url = dotenv!("DATABASE_URL");

    let manager = PostgresConnectionManager::new_from_stringlike(db_url, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    AppState { db_pool: pool }
}

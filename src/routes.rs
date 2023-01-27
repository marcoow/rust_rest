use axum::{
    routing::{get, post},
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use dotenv_codegen::dotenv;
use tokio_postgres::NoTls;
use crate::controllers::users::{create_user, get_user, get_users};

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

pub async fn routes() -> Router {
    let db_url = dotenv!("DATABASE_URL");
    
    let manager = PostgresConnectionManager::new_from_stringlike(db_url, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    Router::new()
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user))
        .with_state(pool)
}

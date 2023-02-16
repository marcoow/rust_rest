use crate::routes::routes;
use crate::state::AppState;
use crate::state::ConnectionPool;
use axum::{body::Body, http::Request, response::Response, Router};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use core::str::FromStr;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::env;
use tokio_postgres::{config::Config, NoTls};
use tower::ServiceExt;

pub async fn setup() -> (Router, ConnectionPool) {
    dotenvy::from_filename(".env.test").unwrap();
    let db_url = env::var("DATABASE_URL").unwrap();
    let config = Config::from_str(&db_url).unwrap();
    let db_name = config.get_dbname().unwrap();

    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let test_db_suffix: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    let test_db_name = format!("{}_{}", db_name, test_db_suffix).to_lowercase();
    client
        .execute(
            &format!("create database {} template {}", test_db_name, db_name),
            &[],
        )
        .await
        .unwrap();

    let mut test_db_config = config.clone();
    test_db_config.dbname(&test_db_name);
    let manager = PostgresConnectionManager::new(test_db_config, NoTls);
    let pool = Pool::builder().build(manager).await.unwrap();

    let app = routes().with_state(AppState {
        db_pool: pool.clone(),
    });

    (app, pool)
}

pub async fn request(app: Router, uri: &str, headers: HashMap<&str, &str>, body: Body) -> Response {
    let mut request_builder = Request::builder().uri(uri);

    for (key, value) in headers {
        request_builder = request_builder.header(key, value);
    }

    let request = request_builder.body(body);

    app.oneshot(request.unwrap()).await.unwrap()
}

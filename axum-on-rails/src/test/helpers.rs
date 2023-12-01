use crate::config::DatabaseConfig;
use axum::{
    body::Body,
    http::{Method, Request},
    response::Response,
    Router,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{ConnectOptions, Connection, Executor, PgPool};
use std::collections::HashMap;
use tower::ServiceExt;
use url::Url;

pub struct TestContext {
    pub app: Router,
    pub db_pool: PgPool,
    db_config: PgConnectOptions,
}

pub fn build_test_context(
    router: Router,
    db_pool: PgPool,
    test_db_config: PgConnectOptions,
) -> TestContext {
    TestContext {
        app: router,
        db_pool,
        db_config: test_db_config,
    }
}

pub async fn prepare_db(config: &DatabaseConfig) -> PgConnectOptions {
    let db_url = Url::parse(&config.url).expect("Invalid DATABASE_URL!");
    let config: PgConnectOptions =
        ConnectOptions::from_url(&db_url).expect("Invalid DATABASE_URL!");
    let db_name = config.get_database().unwrap();

    let root_db_config = config.clone().database("postgres");
    let mut connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();

    let test_db_suffix: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    let test_db_name = format!("{}_{}", db_name, test_db_suffix).to_lowercase();

    let query = format!("CREATE DATABASE {} TEMPLATE {}", test_db_name, db_name);
    connection.execute(query.as_str()).await.unwrap();

    config.clone().database(&test_db_name)
}

pub async fn teardown(context: TestContext) {
    drop(context.app);
    drop(context.db_pool);
    let db_name = context.db_config.get_database().unwrap();
    println!("cleaning up DB, {}", &db_name);
    let root_db_config = context.db_config.clone().database("postgres");

    let mut connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();

    let query = format!("DROP DATABASE IF EXISTS {}", db_name);
    connection.execute(query.as_str()).await.unwrap();
}

pub async fn request(
    app: &Router,
    uri: &str,
    headers: HashMap<&str, &str>,
    body: Body,
    method: Method,
) -> Response {
    let mut request_builder = Request::builder().uri(uri);

    for (key, value) in headers {
        request_builder = request_builder.header(key, value);
    }

    request_builder = request_builder.method(method);

    let request = request_builder.body(body);

    app.clone().oneshot(request.unwrap()).await.unwrap()
}

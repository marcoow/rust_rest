use crate::ConnectionPool;
use axum::{
    body::Body,
    http::{Method, Request},
    response::Response,
    Router,
};
use core::str::FromStr;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::env;
use tokio_postgres::{config::Config, NoTls};
use tower::ServiceExt;

pub struct TestContext {
    pub app: Router,
    pub pool: ConnectionPool,
    db_config: Config,
}

pub fn build_test_context(router: Router, pool: ConnectionPool, test_db_config: Config) -> TestContext {
    TestContext {
        app: router,
        pool,
        db_config: test_db_config,
    }
}

pub async fn prepare_db() -> Config {
    dotenvy::from_filename(".env.test").ok();
    let db_url = env::var("DATABASE_URL").expect("No DATABASE_URL set – cannot run tests!");
    let config = Config::from_str(&db_url).unwrap();
    let db_name = config.get_dbname().unwrap();

    let mut root_db_config = config.clone();
    root_db_config.dbname("postgres");
    let (client, connection) = root_db_config.connect(NoTls).await.unwrap();
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

    test_db_config
}

pub async fn teardown(context: TestContext) {
    drop(context.app);
    drop(context.pool);
    let db_name = context.db_config.get_dbname().unwrap();
    println!("cleaning up DB, {}", &db_name);
    let mut root_db_config = context.db_config.clone();
    root_db_config.dbname("postgres");
    let (client, connection) = root_db_config.connect(NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    println!("we connection1!");

    let result = client
        .execute(&format!("drop database if exists {}", &db_name), &[])
        .await;

    match result {
        Ok(_) => println!("Dropped!"),
        Err(e) => println!("❌ Dropping database {} failed: {:?}!", &db_name, e),
    }
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

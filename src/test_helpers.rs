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
use tokio_postgres::error::SqlState;
use tokio_postgres::{config::Config, NoTls};
use tower::ServiceExt;

static DATABASE_TEMPLATE: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
static DATABASE_TEMPLATE_NAME: &str = "__test_template__";

async fn prepare_db() -> Config {
    dotenvy::from_filename(".env.test").ok();
    let db_url = env::var("DATABASE_URL").expect("No DATABASE_URL set – cannot run tests!");
    let config = Config::from_str(&db_url).unwrap();
    let db_name = config.get_dbname().unwrap();

    DATABASE_TEMPLATE
        .get_or_init(|| {
            let db_url = db_url.clone();
            async move {
                // Create DB template
                {
                    let config = Config::from_str(&db_url).unwrap();
                    let (client, connection) = config.connect(NoTls).await.unwrap();
                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            eprintln!("connection error: {}", e);
                        }
                    });

                    if let Err(e) = client
                        .execute(
                            &format!(
                                "create database {} template {}",
                                DATABASE_TEMPLATE_NAME, db_name
                            ),
                            &[],
                        )
                        .await
                    {
                        if Some(&SqlState::DUPLICATE_DATABASE) != e.code() {
                            client
                                .execute(
                                    &format!(
                                        "alter database {} with is_template FALSE",
                                        DATABASE_TEMPLATE_NAME
                                    ),
                                    &[],
                                )
                                .await
                                .unwrap();
                            client
                                .execute(
                                    &format!("drop database if exists {}", DATABASE_TEMPLATE_NAME),
                                    &[],
                                )
                                .await
                                .unwrap();
                            client
                                .execute(
                                    &format!(
                                        "create database {} template {}",
                                        DATABASE_TEMPLATE_NAME, db_name
                                    ),
                                    &[],
                                )
                                .await
                                .unwrap();
                        }
                    }

                    client
                        .execute(
                            &format!(
                                "alter database {} with is_template TRUE",
                                DATABASE_TEMPLATE_NAME
                            ),
                            &[],
                        )
                        .await
                        .unwrap();
                }
            }
        })
        .await;

    let (client, connection) = config.connect(NoTls).await.unwrap();
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
            &format!(
                "create database {} template {}",
                test_db_name, DATABASE_TEMPLATE_NAME
            ),
            &[],
        )
        .await
        .unwrap();

    let mut test_db_config = config.clone();
    test_db_config.dbname(&test_db_name);

    test_db_config
}

pub async fn setup() -> (Router, ConnectionPool) {
    let test_db_config = prepare_db().await;
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

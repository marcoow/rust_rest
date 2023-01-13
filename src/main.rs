use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio_postgres::NoTls;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let manager = PostgresConnectionManager::new_from_stringlike(
        "postgresql://rust_rest:rust_rest@localhost/rust_rest",
        NoTls,
    )
    .unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    // build our application with a route
    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users", post(create_user))
        .with_state(pool);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

#[derive(Serialize, Debug)]
struct User {
    id: i32,
    username: String,
}

async fn get_users(
    State(pool): State<ConnectionPool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let row = conn
        .query_one("select id, username from users where id = $1", &[&1])
        .await
        .map_err(internal_error)?;

    let id = row.get(0);
    let username = row.get(1);

    let user = User { id, username };

    info!("responding with {:?}", user);

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    Ok(Json(vec![user]))
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

async fn create_user(
    State(pool): State<ConnectionPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let username = payload.username;

    let conn = pool.get().await.map_err(internal_error)?;
    let rows = conn
        .query(
            "insert into users (username) values ($1) returning id",
            &[&username],
        )
        .await
        .map_err(internal_error)?;

    let id = rows[0].get(0);

    let user = User { id, username };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    Ok(Json(user))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

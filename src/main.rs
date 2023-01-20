use axum::{
    extract::Path,
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use tokio_postgres::NoTls;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use validator::Validate;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Vec<String> = env::args().collect();
    let db_url = if args.len() == 2 {
        &args[1]
    } else {
        "postgresql://rust_rest:rust_rest@localhost/rust_rest"
    };

    let manager = PostgresConnectionManager::new_from_stringlike(db_url, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user))
        .with_state(pool);

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
    name: String,
}

async fn get_users(
    State(pool): State<ConnectionPool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let rows = conn
        .query("select id, name from users", &[])
        .await
        .map_err(internal_error)?;

    let users = rows
        .iter()
        .map(|row| User {
            id: row.get(0),
            name: row.get(1),
        })
        .collect();

    info!("responding with {:?}", users);

    Ok(Json(users))
}

async fn get_user(
    State(pool): State<ConnectionPool>,
    Path(id): Path<i32>,
) -> Result<Json<User>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    if let Ok(row) = conn
        .query_one("select id, name from users where id = $1", &[&id])
        .await
    {
        let user = User {
            id: row.get(0),
            name: row.get(1),
        };

        info!("responding with {:?}", user);

        Ok(Json(user))
    } else {
        info!("no user found for id {}", id);

        Err((StatusCode::NOT_FOUND, "".to_string()))
    }
}

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1))]
    name: String,
}

async fn create_user(
    State(pool): State<ConnectionPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    match payload.validate() {
        Ok(_) => {
            let name = payload.name;

            let conn = pool.get().await.map_err(internal_error)?;
            let rows = conn
                .query(
                    "insert into users (name) values ($1) returning id",
                    &[&name],
                )
                .await
                .map_err(internal_error)?;

            let id = rows[0].get(0);

            let user = User { id, name };

            Ok(Json(user))
        }
        Err(err) => Err((StatusCode::UNPROCESSABLE_ENTITY, err.to_string())),
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

use crate::{internal_error, state::AppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

#[derive(Serialize, Debug)]
#[cfg_attr(test, derive(Deserialize))]
pub struct User {
    id: i32,
    name: String,
}

pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = state.db_pool.get().await.map_err(internal_error)?;

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

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<User>, (StatusCode, String)> {
    let conn = state.db_pool.get().await.map_err(internal_error)?;

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
pub struct CreateUser {
    #[validate(length(min = 1))]
    name: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    match payload.validate() {
        Ok(_) => {
            let name = payload.name;

            let conn = state.db_pool.get().await.map_err(internal_error)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use bb8::Pool;
    use bb8_postgres::PostgresConnectionManager;
    use core::str::FromStr;
    use dotenv_codegen::dotenv;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use tokio_postgres::{config::Config, NoTls};
    use tower::ServiceExt;

    type UsersList = Vec<User>;

    #[tokio::test]
    async fn test_get_users() {
        let db_url = dotenv!("DATABASE_URL");
        let config = Config::from_str(db_url).unwrap();
        let db_name = config.get_dbname().unwrap();

        let (client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();
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
        let res = client
            .execute(
                &format!("create database {} template {}", test_db_name, db_name),
                &[],
            )
            .await
            .unwrap();
        println!("{:?}", res);

        let mut test_db_config = config.clone();
        test_db_config.dbname(&test_db_name);
        println!("{:?}", test_db_config);
        let manager = PostgresConnectionManager::new(test_db_config, NoTls);
        let pool = Pool::builder().build(manager).await.unwrap();

        let app = Router::new()
            .route("/users", get(get_users))
            .with_state(AppState {
                db_pool: pool.clone(),
            });

        let conn = pool.get().await.unwrap();

        conn.query(
            "insert into users (name) values ($1) returning id",
            &[&"Test User"],
        )
        .await
        .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let users: UsersList = serde_json::from_slice::<Vec<User>>(&body).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users.get(0).unwrap().name, "Test User");
    }
}

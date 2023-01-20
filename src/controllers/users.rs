use crate::{internal_error, ConnectionPool};
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

pub async fn get_user(
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
pub struct CreateUser {
    #[validate(length(min = 1))]
    name: String,
}

pub async fn create_user(
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use bb8::Pool;
    use bb8_postgres::PostgresConnectionManager;
    use dotenv_codegen::dotenv;
    use tokio_postgres::NoTls;
    use tower::ServiceExt;

    type UsersList = Vec<User>;

    #[tokio::test]
    async fn test_get_users() {
        let db_url = dotenv!("DATABASE_URL");

        let manager = PostgresConnectionManager::new_from_stringlike(db_url, NoTls).unwrap();
        let pool = Pool::builder().build(manager).await.unwrap();

        let app = Router::new()
            .route("/users", get(get_users))
            .with_state(pool.clone());

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

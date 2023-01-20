use crate::{internal_error, ConnectionPool};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

#[derive(Serialize, Debug)]
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

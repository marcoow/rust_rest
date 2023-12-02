use crate::{internal_error, state::AppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use rust_rest_db::entities::Task;
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

pub async fn get_tasks(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let tasks = sqlx::query_as!(Task, "SELECT id, description FROM tasks")
        .fetch_all(&app_state.db_pool)
        .await
        .map_err(internal_error)?;

    info!("responding with {:?}", tasks);

    Ok(Json(tasks))
}

pub async fn get_task(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let task = sqlx::query_as!(Task, "SELECT id, description FROM tasks WHERE id = $1", id)
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(internal_error)?;

    info!("responding with {:?}", task);

    Ok(Json(task))
}

#[derive(Deserialize, Validate)]
#[cfg_attr(test, derive(Serialize))]
pub struct CreateTask {
    #[validate(length(min = 1))]
    description: String,
}

pub async fn create_task(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTask>,
) -> Result<Json<Task>, (StatusCode, String)> {
    match payload.validate() {
        Ok(_) => {
            let description = payload.description;

            let record = sqlx::query!(
                "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
                description
            )
            .fetch_one(&app_state.db_pool)
            .await
            .map_err(internal_error)?;

            let id = record.id;

            let task = Task { id, description };

            Ok(Json(task))
        }
        Err(err) => Err((StatusCode::UNPROCESSABLE_ENTITY, err.to_string())),
    }
}

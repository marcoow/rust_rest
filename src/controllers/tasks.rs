use crate::{internal_error, state::AppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

#[derive(Serialize, Debug)]
#[cfg_attr(test, derive(Deserialize))]
pub struct Task {
    id: i32,
    description: String,
}

pub async fn get_tasks(
    State(state): State<AppState>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let conn = state.db_pool.get().await.map_err(internal_error)?;

    let rows = conn
        .query("select id, description from tasks", &[])
        .await
        .map_err(internal_error)?;

    let tasks = rows
        .iter()
        .map(|row| Task {
            id: row.get(0),
            description: row.get(1),
        })
        .collect();

    info!("responding with {:?}", tasks);

    Ok(Json(tasks))
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let conn = state.db_pool.get().await.map_err(internal_error)?;

    if let Ok(row) = conn
        .query_one("select id, description from tasks where id = $1", &[&id])
        .await
    {
        let task = Task {
            id: row.get(0),
            description: row.get(1),
        };

        info!("responding with {:?}", task);

        Ok(Json(task))
    } else {
        info!("no task found for id {}", id);

        Err((StatusCode::NOT_FOUND, "".to_string()))
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateTask {
    #[validate(length(min = 1))]
    description: String,
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTask>,
) -> Result<Json<Task>, (StatusCode, String)> {
    match payload.validate() {
        Ok(_) => {
            let description = payload.description;

            let conn = state.db_pool.get().await.map_err(internal_error)?;
            let rows = conn
                .query(
                    "insert into tasks (description) values ($1) returning id",
                    &[&description],
                )
                .await
                .map_err(internal_error)?;

            let id = rows[0].get(0);

            let task = Task { id, description };

            Ok(Json(task))
        }
        Err(err) => Err((StatusCode::UNPROCESSABLE_ENTITY, err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{request, setup};
    use axum::body::Body;
    use std::collections::HashMap;

    type TasksList = Vec<Task>;

    #[tokio::test]
    async fn test_get_tasks() {
        let (app, db) = setup().await;

        let conn = db.get().await.unwrap();

        conn.query(
            "insert into tasks (description) values ($1) returning id",
            &[&"Test Task"],
        )
        .await
        .unwrap();

        let response = request(app, "/tasks", HashMap::new(), Body::empty()).await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let tasks: TasksList = serde_json::from_slice::<TasksList>(&body).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks.get(0).unwrap().description, "Test Task");
    }

    #[tokio::test]
    async fn test_get_task() {
        let (app, db) = setup().await;

        let conn = db.get().await.unwrap();

        let rows = conn
            .query(
                "insert into tasks (description) values ($1) returning id",
                &[&"Test Task"],
            )
            .await
            .unwrap();
        let task_id: i32 = rows[0].get(0);

        let response = request(
            app,
            format!("/tasks/{}", task_id).as_str(),
            HashMap::new(),
            Body::empty(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let task: Task = serde_json::from_slice::<Task>(&body).unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.description, "Test Task");
    }
}

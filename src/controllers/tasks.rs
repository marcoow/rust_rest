use crate::{entities::Task, internal_error, state::AppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::helpers::setup;
    use axum::{
        body::Body,
        http::{self, Method},
    };
    use axum_on_rails::test::helpers::{request, teardown, TestContext};
    use axum_on_rails_procs::test;
    use serde_json::json;
    use std::collections::HashMap;

    type TasksList = Vec<Task>;

    #[test]
    async fn test_get_tasks(context: &TestContext) {
        sqlx::query!(
            "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
            "Test Task",
        )
        .fetch_one(&context.db_pool)
        .await
        .unwrap();

        let response = request(
            &context.app,
            "/tasks",
            HashMap::new(),
            Body::empty(),
            Method::GET,
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let tasks: TasksList = serde_json::from_slice::<TasksList>(&body).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks.get(0).unwrap().description, "Test Task");
    }

    #[test]
    async fn test_create_tasks_unauthorized(context: &TestContext) {
        let mut headers = HashMap::new();
        headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");

        let response = request(&context.app, "/tasks", headers, Body::empty(), Method::POST).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    async fn test_create_tasks_authorized(context: &TestContext) {
        sqlx::query!(
            "INSERT INTO users (name, token) VALUES ($1, $2) RETURNING id",
            "Test User",
            "s3kuR t0k3n!",
        )
        .fetch_one(&context.db_pool)
        .await
        .unwrap();

        let mut headers = HashMap::new();
        headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
        headers.insert(http::header::AUTHORIZATION.as_str(), "s3kuR t0k3n!");

        let payload = json!(CreateTask {
            description: String::from("my task")
        });

        let response = request(
            &context.app,
            "/tasks",
            headers,
            Body::from(payload.to_string()),
            Method::POST,
        )
        .await;

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let task: Task = serde_json::from_slice::<Task>(&body).unwrap();
        assert_eq!(task.description, "my task");
    }

    #[test]
    async fn test_get_task(context: &TestContext) {
        let record = sqlx::query!(
            "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
            "Test Task",
        )
        .fetch_one(&context.db_pool)
        .await
        .unwrap();
        let task_id: Uuid = record.id;

        let response = request(
            &context.app,
            format!("/tasks/{}", task_id).as_str(),
            HashMap::new(),
            Body::empty(),
            Method::GET,
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let task: Task = serde_json::from_slice::<Task>(&body).unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.description, "Test Task");
    }
}

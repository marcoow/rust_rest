use axum::{
    body::Body,
    http::{self, Method},
};
use pacesetter::test::helpers::{request, teardown, TestContext};
use pacesetter_procs::test;
use hyper::StatusCode;
use rust_rest_db::entities::Task;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

mod common;

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

    #[derive(Serialize)]
    struct CreateTask {
        description: String,
    }

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

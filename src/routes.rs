use crate::controllers::tasks::{create_task, get_task, get_tasks};
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::validate_request::ValidateRequestHeaderLayer;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tasks", post(create_task))
        .route_layer(ValidateRequestHeaderLayer::bearer("secr3t!"))
        .route("/tasks", get(get_tasks))
        .route("/tasks/:id", get(get_task))
}

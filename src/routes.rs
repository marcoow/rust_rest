use crate::controllers::users::{create_user, get_user, get_users};
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::validate_request::ValidateRequestHeaderLayer;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user))
        .route_layer(ValidateRequestHeaderLayer::bearer("secr3t!"))
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
}

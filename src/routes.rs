use crate::controllers::tasks::{create_task, get_task, get_tasks};
use crate::state::AppState;
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};

#[allow(dead_code)]
#[derive(Clone)]
struct CurrentUser {
    id: i32,
    name: String,
}

async fn auth<B>(
    State(app_state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Ok(conn) = app_state.db_pool.get().await {
        if let Ok(row) = conn
            .query_one(
                "select id, name from users where token = $1",
                &[&auth_header],
            )
            .await
        {
            let current_user = CurrentUser {
                id: row.get(0),
                name: row.get(1),
            };
            req.extensions_mut().insert(current_user);
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/tasks", post(create_task))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .route("/tasks", get(get_tasks))
        .route("/tasks/:id", get(get_task))
        .with_state(app_state)
}

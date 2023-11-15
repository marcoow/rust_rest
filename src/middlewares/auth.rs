use crate::entities::User;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};

#[allow(dead_code)]
#[derive(Clone)]
struct CurrentUser {
    id: i32,
    name: String,
}

pub async fn auth<B>(
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

    if let Ok(user) = sqlx::query_as!(
        User,
        "SELECT id, name FROM users WHERE token = $1",
        auth_header
    )
    .fetch_one(&app_state.sqlx_pool)
    .await
    {
        let current_user = CurrentUser {
            id: user.id,
            name: user.name,
        };
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

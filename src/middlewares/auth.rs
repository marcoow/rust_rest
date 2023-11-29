use crate::entities::User;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::debug;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Clone)]
struct CurrentUser {
    id: Uuid,
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
        debug!("User unauthorized – header missing");
        return Err(StatusCode::UNAUTHORIZED);
    };

    match sqlx::query_as!(
        User,
        "SELECT id, name FROM users WHERE token = $1",
        auth_header
    )
    .fetch_one(&app_state.db_pool)
    .await
    {
        Ok(user) => {
            let current_user = CurrentUser {
                id: user.id,
                name: user.name,
            };
            req.extensions_mut().insert(current_user);
            Ok(next.run(req).await)
        }
        Err(sqlx::Error::RowNotFound) => {
            debug!(r#"User unauthorized – no user for token "{}""#, auth_header);
            Err(StatusCode::UNAUTHORIZED)
        }
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

use axum::{
    extract::State,
    Json,
};
use crate::{
    database::state::AppState,
    routes::auth::AuthUser,
    error::{AppError, ErrorType},
};
use serde_json::json;

// Root handler with authentication check
pub async fn root_handler(
    auth_user: Option<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<AppError>)> {
    match auth_user {
        Some(user) => {
            let response = json!({
                "status": "authenticated",
                "user_role": user.role,
                "redirect": if user.role == "admin" { "/admin" } else { "/dashboard" }
            });
            Ok(Json(response))
        }
        None => {
            let response = json!({
                "status": "unauthenticated",
                "message": "Welcome to LinkSphere API",
                "redirect": "/auth/login"
            });
            Ok(Json(response))
        }
    }
} 
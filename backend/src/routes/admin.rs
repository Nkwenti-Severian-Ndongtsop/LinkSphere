use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::error;

use crate::{
    database::state::AppState,
    models::user::{User, UserRole},
    routes::auth::AuthUser,
    error::{AppError, ErrorType},
};

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub total_links: i64,
    pub total_clicks: i64,
    pub user_role: UserRole,
    pub is_email_verified: bool,
}

#[derive(Serialize)]
pub struct AdminStats {
    users_stats: Vec<UserStats>,
    overall_stats: OverallStats,
}

#[derive(Serialize)]
pub struct OverallStats {
    total_users: i64,
    total_links: i64,
    total_clicks: i64,
    verified_users: i64,
}

// Get all users with their stats
pub async fn get_user_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserStats>>, (StatusCode, Json<AppError>)> {
    let users_with_stats = sqlx::query_as!(
        UserStats,
        r#"
        SELECT 
            u.id,
            u.username,
            u.email,
            u.user_role as "user_role: UserRole",
            u.is_email_verified,
            COALESCE(COUNT(l.id), 0) as "total_links!: i64",
            COALESCE(SUM(l.click_count), 0) as "total_clicks!: i64"
        FROM users u
        LEFT JOIN links l ON u.id = l.user_id
        GROUP BY u.id, u.username, u.email, u.user_role, u.is_email_verified
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to fetch user statistics",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(Json(users_with_stats))
}

// Delete any user (admin only)
pub async fn delete_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
            id, username, email, password_hash,
            user_role as "user_role: UserRole",
            is_email_verified,
            otp_code,
            otp_expires_at,
            verification_token,
            verification_token_expires_at,
            reset_token,
            reset_token_expires_at,
            created_at,
            updated_at
        FROM users WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Database error",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    match user {
        Some(user) if user.user_role == UserRole::Admin => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(AppError::new(
                    "Cannot delete admin user",
                    ErrorType::Forbidden,
                )),
            ));
        }
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(AppError::new(
                    "User not found",
                    ErrorType::NotFound,
                )),
            ));
        }
        _ => {}
    }

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppError::new(
                    "Failed to delete user",
                    ErrorType::DatabaseError,
                )),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Delete any link (admin only)
pub async fn delete_any_link_handler(
    State(state): State<AppState>,
    Path(link_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    sqlx::query!("DELETE FROM links WHERE id = $1", link_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppError::new(
                    "Failed to delete link",
                    ErrorType::DatabaseError,
                )),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct LinkWithUser {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub description: String,
    pub click_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uploader_username: String,
    pub user_id: Uuid,
    pub favicon_url: Option<String>,
}

// Get all links across all users
pub async fn get_all_links_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<LinkWithUser>>, (StatusCode, Json<AppError>)> {
    let links = sqlx::query_as!(
        LinkWithUser,
        r#"
        SELECT 
            l.id,
            l.title,
            l.url,
            l.description,
            l.click_count,
            l.created_at,
            l.updated_at,
            l.favicon_url,
            u.username as uploader_username,
            u.id as user_id
        FROM links l
        JOIN users u ON l.user_id = u.id
        ORDER BY l.created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to fetch links",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(Json(links))
} 
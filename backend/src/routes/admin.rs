use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use tracing::{info, error};
use uuid::Uuid;

use crate::{
    database::state::AppState,
    models::user::{User, UserRole},
    routes::auth::AuthUser,
};

#[derive(Serialize)]
pub struct UserStats {
    id: Uuid,
    username: String,
    email: String,
    total_links: i64,
    total_clicks: i64,
    is_email_verified: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct AdminStats {
    total_users: i64,
    total_links: i64,
    total_clicks: i64,
    users: Vec<UserStats>,
}

// Get all users with their stats
pub async fn get_user_stats_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<AdminStats>, (StatusCode, Json<serde_json::Value>)> {
    // Verify admin status
    if !auth_user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Admin access required"
            }))
        ));
    }

    let users_with_stats = sqlx::query_as!(
        UserStats,
        r#"
        SELECT 
            u.id,
            u.username,
            u.email,
            u.is_email_verified,
            u.created_at,
            COUNT(DISTINCT l.id) as total_links,
            COALESCE(SUM(l.click_count), 0) as total_clicks
        FROM users u
        LEFT JOIN links l ON u.id = l.user_id
        GROUP BY u.id, u.username, u.email, u.is_email_verified, u.created_at
        ORDER BY u.created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to fetch user stats"
            }))
        )
    })?;

    // Get overall statistics
    let overall_stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT l.id) as total_links,
            COALESCE(SUM(l.click_count), 0) as total_clicks
        FROM users u
        LEFT JOIN links l ON u.id = l.user_id
        "#
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to fetch overall stats"
            }))
        )
    })?;

    Ok(Json(AdminStats {
        total_users: overall_stats.total_users.unwrap_or(0),
        total_links: overall_stats.total_links.unwrap_or(0),
        total_clicks: overall_stats.total_clicks.unwrap_or(0),
        users: users_with_stats,
    }))
}

// Delete any user (admin only)
pub async fn delete_user_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Verify admin status
    if !auth_user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Admin access required"
            }))
        ));
    }

    // Don't allow deleting the admin account
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to fetch user"
            }))
        )
    })?;

    if let Some(user) = user {
        if user.user_role == UserRole::Admin {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "Cannot delete admin account"
                }))
            ));
        }
    }

    // Delete user and their links (cascade should handle this)
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to delete user"
                }))
            )
        })?;

    info!("Admin {} deleted user {}", auth_user.user_id, user_id);
    Ok(StatusCode::NO_CONTENT)
}

// Delete any link (admin only)
pub async fn delete_any_link_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(link_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Verify admin status
    if !auth_user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Admin access required"
            }))
        ));
    }

    sqlx::query!("DELETE FROM links WHERE id = $1", link_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to delete link"
                }))
            )
        })?;

    info!("Admin {} deleted link {}", auth_user.user_id, link_id);
    Ok(StatusCode::NO_CONTENT)
}

// Get all links across all users
pub async fn get_all_links_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<LinkWithUser>>, (StatusCode, Json<serde_json::Value>)> {
    // Verify admin status
    if !auth_user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Admin access required"
            }))
        ));
    }

    let links = sqlx::query_as!(
        LinkWithUser,
        r#"
        SELECT 
            l.id,
            l.url,
            l.title,
            l.description,
            l.click_count,
            l.created_at,
            u.username as user_username,
            u.email as user_email
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
            Json(serde_json::json!({
                "error": "Failed to fetch links"
            }))
        )
    })?;

    Ok(Json(links))
}

#[derive(Serialize)]
pub struct LinkWithUser {
    id: Uuid,
    url: String,
    title: String,
    description: String,
    click_count: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    user_username: String,
    user_email: String,
} 
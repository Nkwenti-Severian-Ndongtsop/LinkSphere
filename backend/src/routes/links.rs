use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{postgres::PgRow, FromRow, Row};
use crate::{
    database::state::AppState,
    error::{AppError, ErrorType},
    routes::auth::AuthUser,
    models::user::{User, UserRole},
};
use chrono::{DateTime, Utc};
use tracing::error;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Link {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: String,
    pub description: String,
    pub click_count: i32,
    pub favicon_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uploader_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub url: String,
    pub title: String,
    pub description: String,
    pub favicon_url: Option<String>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub favicon_url: Option<String>,
}

// Create a new link
pub async fn create_link_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateLinkRequest>,
) -> Result<Json<Link>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();
    let link = sqlx::query_as::<_, Link>(
        r#"
        INSERT INTO links (user_id, url, title, description, favicon_url)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, url, title, description, click_count, favicon_url, created_at, updated_at,
            (SELECT username FROM users WHERE id = $1) as uploader_name
        "#
    )
    .bind(user_id)
    .bind(&payload.url)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.favicon_url)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to create link",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    // Add tags and categories
    for tag_name in payload.tags {
        let tag_id = Uuid::parse_str(&tag_name).unwrap();
        sqlx::query!(
            "INSERT INTO link_tags (link_id, tag_id) VALUES ($1, $2)",
            link.id,
            tag_id
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppError::new(
                    "Failed to add tags",
                    ErrorType::DatabaseError,
                )),
            )
        })?;
    }

    for category_name in payload.categories {
        let category_id = Uuid::parse_str(&category_name).unwrap();
        sqlx::query!(
            "INSERT INTO link_categories (link_id, category_id) VALUES ($1, $2)",
            link.id,
            category_id
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppError::new(
                    "Failed to add categories",
                    ErrorType::DatabaseError,
                )),
            )
        })?;
    }

    Ok(Json(link))
}

// Get all links for current user
pub async fn get_user_links_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Link>>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();
    let links = sqlx::query_as!(
        Link,
        r#"
        SELECT 
            l.id,
            l.user_id,
            l.url,
            l.title,
            l.description,
            l.click_count,
            l.favicon_url,
            l.created_at,
            l.updated_at,
            u.username as uploader_name
        FROM links l
        JOIN users u ON l.user_id = u.id
        WHERE l.user_id = $1
        ORDER BY l.created_at DESC
        "#,
        user_id
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

// Update a link
pub async fn update_link_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(link_id): Path<Uuid>,
    Json(payload): Json<UpdateLinkRequest>,
) -> Result<Json<Link>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    let link = sqlx::query_as::<_, Link>(
        r#"
        UPDATE links
        SET 
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            url = COALESCE($3, url),
            favicon_url = $4,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $5 AND user_id = $6
        RETURNING id, user_id, url, title, description, click_count, favicon_url, created_at, updated_at,
            (SELECT username FROM users WHERE id = $6) as uploader_name
        "#
    )
    .bind(payload.title.as_deref())
    .bind(payload.description.as_deref())
    .bind(payload.url.as_deref())
    .bind(payload.favicon_url.as_deref())
    .bind(link_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to update link",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    match link {
        Some(link) => Ok(Json(link)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        )),
    }
}

// Delete a link
pub async fn delete_link_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(link_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    // Check if user owns the link or is admin
    if !auth_user.is_admin {
        let link = sqlx::query!(
            "SELECT user_id FROM links WHERE id = $1",
            link_id
        )
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppError::new(
                    "Failed to check link ownership",
                    ErrorType::DatabaseError,
                )),
            )
        })?;

        match link {
            Some(link) if link.user_id != Uuid::parse_str(&auth_user.user_id).unwrap() => {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(AppError::new(
                        "You don't have permission to delete this link",
                        ErrorType::Forbidden,
                    )),
                ));
            }
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(AppError::new(
                        "Link not found",
                        ErrorType::NotFound,
                    )),
                ));
            }
            _ => {}
        }
    }

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

// Increment click count
pub async fn increment_click_count_handler(
    State(state): State<AppState>,
    Path(link_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    sqlx::query!(
        "UPDATE links SET click_count = click_count + 1 WHERE id = $1",
        link_id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to increment click count",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(StatusCode::OK)
}

// Get link tags
pub async fn get_link_tags_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(link_id): Path<Uuid>,
) -> Result<Json<Vec<Tag>>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.id, t.name, t.user_id
        FROM tags t
        JOIN link_tags lt ON t.id = lt.tag_id
        JOIN links l ON lt.link_id = l.id
        WHERE l.id = $1 AND l.user_id = $2
        "#
    )
    .bind(link_id)
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to fetch link tags: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(Json(tags))
}

// Add tag to link
pub async fn add_link_tag_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((link_id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    // Verify link ownership
    let link = sqlx::query(
        "SELECT id FROM links WHERE id = $1 AND user_id = $2"
    )
    .bind(link_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Database error: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if link.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        ));
    }

    // Add tag to link
    sqlx::query(
        r#"
        INSERT INTO link_tags (link_id, tag_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(link_id)
    .bind(tag_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to add tag to link: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// Remove tag from link
pub async fn remove_link_tag_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((link_id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    // Verify link ownership
    let link = sqlx::query(
        "SELECT id FROM links WHERE id = $1 AND user_id = $2"
    )
    .bind(link_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Database error: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if link.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        ));
    }

    // Remove tag from link
    let result = sqlx::query(
        r#"
        DELETE FROM link_tags
        WHERE link_id = $1 AND tag_id = $2
        "#
    )
    .bind(link_id)
    .bind(tag_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to remove tag from link: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Tag not found on link",
                ErrorType::NotFound,
            )),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Get all public links without authentication
pub async fn get_public_links_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<Link>>, (StatusCode, Json<AppError>)> {
    let links = sqlx::query_as!(
        Link,
        r#"
        SELECT 
            l.id,
            l.user_id,
            l.url,
            l.title,
            l.description,
            l.click_count,
            l.favicon_url,
            l.created_at,
            l.updated_at,
            u.username as uploader_name
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
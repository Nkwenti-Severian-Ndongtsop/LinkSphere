use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{
    database::state::AppState,
    error::{AppError, ErrorType},
    routes::auth::AuthUser,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: String,
    pub description: String,
    pub click_count: i32,
    pub favicon_url: Option<String>,
    pub uploader_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub url: String,
    pub title: String,
    pub description: String,
    pub favicon_url: Option<String>,
    pub tags: Option<Vec<Uuid>>,
    pub categories: Option<Vec<Uuid>>,
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
    
    // Start transaction
    let mut tx = state.pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to start transaction",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    // Create link
    let link = sqlx::query_as!(
        Link,
        r#"
        INSERT INTO links (user_id, url, title, description, favicon_url, click_count, uploader_name)
        VALUES ($1, $2, $3, $4, $5, 0, (SELECT username FROM users WHERE id = $1))
        RETURNING id, user_id, url, title, description, click_count, favicon_url, uploader_name
        "#,
        user_id,
        payload.url,
        payload.title,
        payload.description,
        payload.favicon_url,
    )
    .fetch_one(&mut tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to create link",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    // Add tags if provided
    if let Some(tags) = payload.tags {
        for tag_id in tags {
            sqlx::query!(
                r#"
                INSERT INTO link_tags (link_id, tag_id)
                VALUES ($1, $2)
                "#,
                link.id,
                tag_id
            )
            .execute(&mut tx)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AppError::new(
                        "Failed to add tags",
                        ErrorType::DatabaseError,
                    )),
                )
            })?;
        }
    }

    // Add categories if provided
    if let Some(categories) = payload.categories {
        for category_id in categories {
            sqlx::query!(
                r#"
                INSERT INTO link_categories (link_id, category_id)
                VALUES ($1, $2)
                "#,
                link.id,
                category_id
            )
            .execute(&mut tx)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AppError::new(
                        "Failed to add categories",
                        ErrorType::DatabaseError,
                    )),
                )
            })?;
        }
    }

    // Commit transaction
    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to commit transaction",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(Json(link))
}

// Get all links for current user
pub async fn get_links_handler(
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
    .map_err(|_| {
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

    let link = sqlx::query_as!(
        Link,
        r#"
        UPDATE links
        SET 
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            url = COALESCE($3, url),
            favicon_url = $4
        WHERE id = $5 AND user_id = $6
        RETURNING id, user_id, url, title, description, click_count, favicon_url, uploader_name
        "#,
        payload.title,
        payload.description,
        payload.url,
        payload.favicon_url,
        link_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
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
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    let result = sqlx::query!(
        r#"
        DELETE FROM links
        WHERE id = $1 AND user_id = $2
        "#,
        link_id,
        user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to delete link",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Increment click count
pub async fn increment_click_count_handler(
    State(state): State<AppState>,
    Path(link_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let result = sqlx::query!(
        r#"
        UPDATE links
        SET click_count = click_count + 1
        WHERE id = $1
        "#,
        link_id
    )
    .execute(&state.pool)
    .await
        .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to increment click count",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Get link tags
pub async fn get_link_tags_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(link_id): Path<Uuid>,
) -> Result<Json<Vec<Tag>>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    let tags = sqlx::query_as!(
        Tag,
        r#"
        SELECT t.id, t.name, t.user_id
        FROM tags t
        JOIN link_tags lt ON t.id = lt.tag_id
        JOIN links l ON lt.link_id = l.id
        WHERE l.id = $1 AND l.user_id = $2
        "#,
        link_id,
        user_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to fetch link tags",
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
    let link = sqlx::query!(
        "SELECT id FROM links WHERE id = $1 AND user_id = $2",
        link_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Database error",
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
    sqlx::query!(
        r#"
        INSERT INTO link_tags (link_id, tag_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
        link_id,
        tag_id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to add tag to link",
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
    let link = sqlx::query!(
        "SELECT id FROM links WHERE id = $1 AND user_id = $2",
        link_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Database error",
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
    let result = sqlx::query!(
        r#"
        DELETE FROM link_tags
        WHERE link_id = $1 AND tag_id = $2
        "#,
        link_id,
        tag_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to remove tag from link",
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
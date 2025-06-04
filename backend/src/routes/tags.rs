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
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

// Get all tags for the current user
pub async fn get_tags_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Tag>>, (StatusCode, Json<AppError>)> {
    let tags = sqlx::query_as!(
        Tag,
        r#"
        SELECT id, name, user_id
        FROM tags
        WHERE user_id = $1
        ORDER BY name
        "#,
        Uuid::parse_str(&auth_user.user_id).unwrap()
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to fetch tags",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(Json(tags))
}

// Create a new tag
pub async fn create_tag_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateTagRequest>,
) -> Result<Json<Tag>, (StatusCode, Json<AppError>)> {
    // Validate tag name
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AppError::new(
                "Tag name cannot be empty",
                ErrorType::ValidationError,
            )),
        ));
    }

    let tag = sqlx::query_as!(
        Tag,
        r#"
        INSERT INTO tags (name, user_id)
        VALUES ($1, $2)
        RETURNING id, name, user_id
        "#,
        payload.name,
        Uuid::parse_str(&auth_user.user_id).unwrap()
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique constraint") {
            (
                StatusCode::CONFLICT,
                Json(AppError::new(
                    "Tag already exists",
                    ErrorType::ValidationError,
                )),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppError::new(
                    "Failed to create tag",
                    ErrorType::DatabaseError,
                )),
            )
        }
    })?;

    Ok(Json(tag))
}

// Delete a tag
pub async fn delete_tag_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(tag_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let result = sqlx::query!(
        r#"
        DELETE FROM tags
        WHERE id = $1 AND user_id = $2
        "#,
        tag_id,
        Uuid::parse_str(&auth_user.user_id).unwrap()
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to delete tag",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Tag not found",
                ErrorType::NotFound,
            )),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
} 
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
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
}

// Get all categories for the current user
pub async fn get_categories_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Category>>, (StatusCode, Json<AppError>)> {
    let categories = sqlx::query_as!(
        Category,
        r#"
        SELECT id, name, description, user_id
        FROM categories
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
                "Failed to fetch categories",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(Json(categories))
}

// Create a new category
pub async fn create_category_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Json<Category>, (StatusCode, Json<AppError>)> {
    // Validate category name
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AppError::new(
                "Category name cannot be empty",
                ErrorType::ValidationError,
            )),
        ));
    }

    let category = sqlx::query_as!(
        Category,
        r#"
        INSERT INTO categories (name, description, user_id)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, user_id
        "#,
        payload.name,
        payload.description,
        Uuid::parse_str(&auth_user.user_id).unwrap()
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique constraint") {
            (
                StatusCode::CONFLICT,
                Json(AppError::new(
                    "Category already exists",
                    ErrorType::ValidationError,
                )),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AppError::new(
                    "Failed to create category",
                    ErrorType::DatabaseError,
                )),
            )
        }
    })?;

    Ok(Json(category))
}

// Delete a category
pub async fn delete_category_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(category_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let result = sqlx::query!(
        r#"
        DELETE FROM categories
        WHERE id = $1 AND user_id = $2
        "#,
        category_id,
        Uuid::parse_str(&auth_user.user_id).unwrap()
    )
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to delete category",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Category not found",
                ErrorType::NotFound,
            )),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
} 
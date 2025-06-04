use axum::{
    extract::{State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use crate::{
    database::state::AppState,
    error::{AppError, ErrorType},
    routes::auth::AuthUser,
    services::password::PasswordService,
};

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_email_verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, max = 100))]
    pub new_password: String,
}

// Get current user's profile
pub async fn get_profile_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserProfile>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    let profile = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT id, username, email, is_email_verified, created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to fetch user profile",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    match profile {
        Some(profile) => Ok(Json(profile)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "User not found",
                ErrorType::NotFound,
            )),
        )),
    }
}

// Update user profile
pub async fn update_profile_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, (StatusCode, Json<AppError>)> {
    // Validate payload
    if let Err(_) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AppError::new(
                "Validation error",
                ErrorType::ValidationError,
            )),
        ));
    }

    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    // Check if username or email is already taken
    if let Some(username) = &payload.username {
        let existing = sqlx::query!(
            "SELECT id FROM users WHERE username = $1 AND id != $2",
            username,
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

        if existing.is_some() {
            return Err((
                StatusCode::CONFLICT,
                Json(AppError::new(
                    "Username already taken",
                    ErrorType::ValidationError,
                )),
            ));
        }
    }

    if let Some(email) = &payload.email {
        let existing = sqlx::query!(
            "SELECT id FROM users WHERE email = $1 AND id != $2",
            email,
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

        if existing.is_some() {
            return Err((
                StatusCode::CONFLICT,
                Json(AppError::new(
                    "Email already taken",
                    ErrorType::ValidationError,
                )),
            ));
        }
    }

    // Update profile
    let profile = sqlx::query_as!(
        UserProfile,
        r#"
        UPDATE users
        SET 
            username = COALESCE($1, username),
            email = COALESCE($2, email),
            is_email_verified = CASE WHEN $2 IS NOT NULL THEN false ELSE is_email_verified END
        WHERE id = $3
        RETURNING id, username, email, is_email_verified, created_at
        "#,
        payload.username,
        payload.email,
        user_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to update profile",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(Json(profile))
}

// Change password
pub async fn change_password_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    // Validate payload
    if let Err(_) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AppError::new(
                "Validation error",
                ErrorType::ValidationError,
            )),
        ));
    }

    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    // Get current password hash
    let current_hash = sqlx::query!(
        "SELECT password_hash FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Database error",
                ErrorType::DatabaseError,
            )),
        )
    })?
    .password_hash;

    // Verify current password
    if !PasswordService::verify_password(&payload.current_password, &current_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Password verification error",
                ErrorType::ServerError,
            )),
        )
    })? {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AppError::new(
                "Current password is incorrect",
                ErrorType::ValidationError,
            )),
        ));
    }

    // Hash new password
    let new_hash = PasswordService::hash_password(&payload.new_password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Password hashing error",
                ErrorType::ServerError,
            )),
        )
    })?;

    // Update password
    sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        new_hash,
        user_id
    )
    .execute(&state.pool)
    .await
        .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to update password",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
} 
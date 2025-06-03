use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::database::state::AppState;
use crate::models::user::{User, CreateUserDto, UserRole};
use sqlx::types::chrono::Utc;

pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<Json<User>, StatusCode> {
    // In a real application, you would hash the password here
    // For this example, we'll just store it as is (NOT recommended for production!)
    let password_hash = payload.password;

    let query = r#"
        INSERT INTO users (username, email, password_hash, user_role, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, username, email, password_hash, user_role, created_at, updated_at
    "#;

    let now = Utc::now();

    match sqlx::query_as::<_, User>(query)
        .bind(payload.username)
        .bind(payload.email)
        .bind(password_hash)
        .bind(UserRole::User)
        .bind(now)
        .bind(now)
        .fetch_one(&state.pool)
        .await
    {
        Ok(user) => {
            println!("Created new user with ID: {}", user.id);
            Ok(Json(user))
        }
        Err(e) => {
            eprintln!("Error creating user: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
} 
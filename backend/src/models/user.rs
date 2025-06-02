use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Serialize, FromRow, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub is_admin: bool,
} 
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use crate::models::user::UserRole;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub user_role: UserRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otp_required: Option<bool>,
}

#[derive(Deserialize)]
pub struct OtpVerificationRequest {
    pub email: String,
    pub otp_code: String,
}

#[derive(Serialize)]
pub struct OtpResponse {
    pub message: String,
}

#[derive(sqlx::FromRow)]
pub struct OtpRecord {
    pub id: i32,
    pub user_id: i32,
    pub email: String,
    pub otp_code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub verified: bool,
}

impl OtpRecord {
    pub fn new(user_id: i32, email: String, otp_code: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            user_id,
            email,
            otp_code,
            created_at: now,
            expires_at: now + Duration::minutes(10), // OTP expires in 10 minutes
            verified: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub message: String,
    pub error_type: String,
} 
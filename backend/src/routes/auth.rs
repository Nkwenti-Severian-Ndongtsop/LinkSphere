use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    extract::TypedHeader,
    headers::{authorization::Bearer, Authorization},
};
use tower_cookies::{Cookie, cookie::{CookieJar, SameSite}, Cookies};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{env, time::{SystemTime, UNIX_EPOCH}};
use validator::{Validate, ValidationError};
use regex::Regex;

use crate::{
    database::state::AppState,
    models::user::User,
};

// JWT configuration
const JWT_EXPIRATION: u64 = 24 * 60 * 60;
const COOKIE_EXPIRATION: u64 = JWT_EXPIRATION;

// Validation constants
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 100;
const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 50;
const USERNAME_PATTERN: &str = r"^[a-zA-Z0-9_-]+$";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub nbf: u64,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum AuthErrorType {
    InvalidToken,
    MissingToken,
    ServerError,
    ValidationError,
}

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub message: String,
    pub error_type: AuthErrorType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self.error_type {
            AuthErrorType::InvalidToken | AuthErrorType::MissingToken => StatusCode::UNAUTHORIZED,
            AuthErrorType::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthErrorType::ValidationError => StatusCode::BAD_REQUEST,
        };
        (status, Json(self)).into_response()
    }
}

// Authentication extractor for protected routes
pub struct AuthUser {
    pub user_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // First try to extract the token from the Authorization header
        let auth_header = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
            .await
            .map(|header| header.token().to_string());

        let token = match auth_header {
            Ok(token) => token,
            Err(_) => {
                // If no Authorization header, try to get the token from cookies
                let cookie_jar = parts.extensions.get::<CookieJar>().ok_or_else(|| AuthError {
                    message: "No authentication token found in header or cookies".to_string(),
                    error_type: AuthErrorType::MissingToken,
                    details: None,
                })?;

                cookie_jar
                    .get("auth_token")
                    .map(|cookie| cookie.value().to_string())
                    .ok_or_else(|| AuthError {
                        message: "No authentication token found".to_string(),
                        error_type: AuthErrorType::MissingToken,
                        details: None,
                    })?
            }
        };

        let claims = verify_token(&token)
            .map_err(|e| AuthError {
                message: e.to_string(),
                error_type: AuthErrorType::InvalidToken,
                details: None,
            })?;

        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}

// Token generation and verification
pub fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + JWT_EXPIRATION,
        iat: now,
        nbf: now,
    };

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    let mut validation = Validation::default();
    validation.leeway = 60; // 1 minute leeway for clock drift
    validation.validate_exp = true;
    validation.validate_nbf = true;
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )?;
    
    Ok(token_data.claims)
}

// Password hashing and verification
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// Response type for authentication
#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    id: i32,
    username: String,
    email: String,
    is_admin: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
        }
    }
}

// Helper function to create an authentication cookie
pub fn create_auth_cookie(token: &str) -> Cookie<'static> {
    let mut cookie = Cookie::new("auth_token", token.to_string());
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_max_age(tower_cookies::cookie::time::Duration::seconds(COOKIE_EXPIRATION as i64));
    cookie
}

// Helper function to create a cookie that invalidates the auth token
pub fn create_logout_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::new("auth_token", "");
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_max_age(tower_cookies::cookie::time::Duration::seconds(0));
    cookie
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,
    #[validate(
        length(
            min = "MIN_PASSWORD_LENGTH",
            max = "MAX_PASSWORD_LENGTH",
            message = "Password must be between 8 and 100 characters"
        )
    )]
    password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,
    #[validate(
        length(
            min = "MIN_PASSWORD_LENGTH",
            max = "MAX_PASSWORD_LENGTH",
            message = "Password must be between 8 and 100 characters"
        ),
        custom = "validate_password_strength"
    )]
    password: String,
    #[validate(
        length(
            min = "MIN_USERNAME_LENGTH",
            max = "MAX_USERNAME_LENGTH",
            message = "Username must be between 3 and 50 characters"
        ),
        regex(
            path = "USERNAME_REGEX",
            message = "Username can only contain letters, numbers, underscores, and hyphens"
        ),
        custom = "validate_username_reserved"
    )]
    username: String,
}

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(USERNAME_PATTERN).unwrap();
    static ref RESERVED_USERNAMES: Vec<&'static str> = vec![
        "admin", "administrator", "root", "system", "moderator",
        "mod", "support", "help", "info", "contact", "test",
        "guest", "anonymous", "user"
    ];
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    // Check for at least one uppercase letter
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("Password must contain at least one uppercase letter"));
    }

    // Check for at least one lowercase letter
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::new("Password must contain at least one lowercase letter"));
    }

    // Check for at least one number
    if !password.chars().any(|c| c.is_numeric()) {
        return Err(ValidationError::new("Password must contain at least one number"));
    }

    // Check for at least one special character
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::new("Password must contain at least one special character"));
    }

    Ok(())
}

fn validate_username_reserved(username: &str) -> Result<(), ValidationError> {
    if RESERVED_USERNAMES.contains(&username.to_lowercase().as_str()) {
        return Err(ValidationError::new("This username is reserved"));
    }
    Ok(())
}

pub async fn login_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<AuthError>)> {
    // Validate input
    if let Err(validation_errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthError {
                message: "Validation error".to_string(),
                error_type: AuthErrorType::ValidationError,
                details: Some(validation_errors.to_string()),
            }),
        ));
    }

    // Find user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthError {
                    message: "Database error".to_string(),
                    error_type: AuthErrorType::ServerError,
                    details: Some(e.to_string()),
                }),
            )
        })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                message: "Invalid credentials".to_string(),
                error_type: AuthErrorType::InvalidToken,
                details: None,
            }),
        )
    })?;

    // Verify password
    let is_valid = verify_password(&payload.password, &user.password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthError {
                message: "Password verification error".to_string(),
                error_type: AuthErrorType::ServerError,
                details: Some(e.to_string()),
            }),
        )
    })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                message: "Invalid credentials".to_string(),
                error_type: AuthErrorType::InvalidToken,
                details: None,
            }),
        ));
    }

    // Generate JWT token
    let token = generate_token(&user.id.to_string()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthError {
                message: "Token generation error".to_string(),
                error_type: AuthErrorType::ServerError,
                details: Some(e.to_string()),
            }),
        )
    })?;

    // Set auth cookie
    cookies.add(create_auth_cookie(&token));

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<AuthError>)> {
    // Validate input
    if let Err(validation_errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthError {
                message: "Validation error".to_string(),
                error_type: AuthErrorType::ValidationError,
                details: Some(validation_errors.to_string()),
            }),
        ));
    }

    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 OR username = $2",
    )
    .bind(&payload.email)
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthError {
                message: "Database error".to_string(),
                error_type: AuthErrorType::ServerError,
                details: Some(e.to_string()),
            }),
        )
    })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthError {
                message: "User already exists".to_string(),
                error_type: AuthErrorType::ValidationError,
                details: None,
            }),
        ));
    }

    // Hash password
    let password_hash = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthError {
                message: "Password hashing error".to_string(),
                error_type: AuthErrorType::ServerError,
                details: Some(e.to_string()),
            }),
        )
    })?;

    // Create user
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash, is_admin, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) 
         RETURNING *",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(false) // New users are not admins by default
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthError {
                message: "Failed to create user".to_string(),
                error_type: AuthErrorType::ServerError,
                details: Some(e.to_string()),
            }),
        )
    })?;

    // Generate JWT token
    let token = generate_token(&user.id.to_string()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthError {
                message: "Token generation error".to_string(),
                error_type: AuthErrorType::ServerError,
                details: Some(e.to_string()),
            }),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn logout_handler(
    cookies: Cookies,
) -> Result<(), (StatusCode, Json<AuthError>)> {
    cookies.add(create_logout_cookie());
    Ok(())
} 
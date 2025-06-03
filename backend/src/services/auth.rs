use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sqlx::PgPool;

use crate::models::user::{User, CreateUserDto, LoginDto, UserRole};
use crate::error::AppError;

const JWT_SECRET: &[u8] = b"your-secret-key"; // In production, use environment variable
const ACCESS_TOKEN_DURATION: i64 = 15; // 15 minutes
const REFRESH_TOKEN_DURATION: i64 = 7 * 24 * 60; // 7 days in minutes

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register(&self, dto: CreateUserDto) -> Result<(User, AuthTokens), AppError> {
        // Check if email already exists
        if let Some(_) = User::find_by_email(&self.pool, &dto.email).await? {
            return Err(AppError::EmailAlreadyExists);
        }

        let user = User::create(&self.pool, dto).await?;
        let tokens = self.generate_tokens(&user)?;

        // TODO: Send verification email
        
        Ok((user, tokens))
    }

    pub async fn login(&self, dto: LoginDto) -> Result<(User, AuthTokens), AppError> {
        let user = User::find_by_email(&self.pool, &dto.email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        if !user.verify_password(&dto.password) {
            return Err(AppError::InvalidCredentials);
        }

        if !user.is_email_verified {
            return Err(AppError::EmailNotVerified);
        }

        let tokens = self.generate_tokens(&user)?;
        Ok((user, tokens))
    }

    pub async fn verify_email(&self, token: &str) -> Result<bool, AppError> {
        let verified = User::verify_email(&self.pool, token).await?;
        if !verified {
            return Err(AppError::InvalidToken);
        }
        Ok(true)
    }

    pub async fn create_password_reset(&self, email: &str) -> Result<Option<String>, AppError> {
        let token = User::create_password_reset(&self.pool, email).await?;
        // TODO: Send password reset email if token is Some
        Ok(token)
    }

    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<bool, AppError> {
        let reset = User::reset_password(&self.pool, token, new_password).await?;
        if !reset {
            return Err(AppError::InvalidToken);
        }
        Ok(true)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthTokens, AppError> {
        let claims = self.verify_token(refresh_token)?;
        
        let user = sqlx::query_as!(
            User,
            r#"SELECT * FROM users WHERE id = $1"#,
            claims.sub
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::InvalidToken)?;

        self.generate_tokens(&user)
    }

    fn generate_tokens(&self, user: &User) -> Result<AuthTokens, AppError> {
        let now = Utc::now();
        
        let access_claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            exp: (now + Duration::minutes(ACCESS_TOKEN_DURATION)).timestamp(),
        };

        let refresh_claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            exp: (now + Duration::minutes(REFRESH_TOKEN_DURATION)).timestamp(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(JWT_SECRET),
        )?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(JWT_SECRET),
        )?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::new(Algorithm::HS256),
        )?;

        Ok(claims.claims)
    }
} 
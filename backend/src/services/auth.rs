#![allow(dead_code)]

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;
use sqlx::PgPool;
use std::env;

use crate::models::user::{User, CreateUserDto, LoginDto, UserRole};
use crate::error::{AppError};

// In production, use environment variable
const ACCESS_TOKEN_DURATION: i64 = 15; // 15 minutes
const REFRESH_TOKEN_DURATION: i64 = 7 * 24 * 60; // 7 days in minutes

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub email: String,
    pub user_role: UserRole,
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

    pub async fn register(&self, dto: CreateUserDto) -> Result<User, AppError> {
        // Check if email already exists
        if let Some(_) = User::find_by_email(&self.pool, &dto.email).await? {
            return Err(AppError::email_already_exists());
        }

        // Create user
        let user = User::create(&self.pool, dto).await?;
        Ok(user)
    }

    pub async fn login(&self, dto: LoginDto) -> Result<AuthTokens, AppError> {
        let user = User::find_by_email(&self.pool, &dto.email)
            .await?
            .ok_or_else(|| AppError::invalid_credentials())?;

        // Verify password
        if !user.verify_password(&dto.password).unwrap_or(false) {
            return Err(AppError::invalid_credentials());
        }

        // Check if email is verified
        if !user.is_email_verified {
            return Err(AppError::email_not_verified());
        }

        // Generate tokens
        let tokens = self.generate_tokens(&user)?;
        Ok(tokens)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthTokens, AppError> {
        let claims = self.verify_token(refresh_token)?;
        
        // Check if user still exists and is active
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash,
                user_role as "user_role: UserRole",
                is_email_verified,
                otp_code,
                otp_expires_at,
                verification_token,
                verification_token_expires_at,
                reset_token,
                reset_token_expires_at,
                created_at,
                updated_at
            FROM users WHERE id = $1
            "#,
            claims.sub
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::invalid_token())?;
    
    // Generate new tokens
    let tokens = self.generate_tokens(&user)?;
    Ok(tokens)
}

pub async fn verify_access_token(&self, token: &str) -> Result<Claims, AppError> {
    let claims = self.verify_token(token)?;
    
    // Check if user still exists and is active
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
        id, username, email, password_hash,
        user_role as "user_role: UserRole",
        is_email_verified,
        otp_code,
        otp_expires_at,
        verification_token,
                verification_token_expires_at,
                reset_token,
                reset_token_expires_at,
                created_at,
                updated_at
            FROM users WHERE id = $1
            "#,
            claims.sub
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::invalid_token())?;
        
    Ok(Claims {
        sub: user.id,
            email: user.email,
            user_role: user.user_role,
            exp: claims.exp,
        })
    }

    fn generate_tokens(&self, user: &User) -> Result<AuthTokens, AppError> {
        let now = Utc::now();
        let jwt_secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let access_claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            user_role: user.user_role,
            exp: (now + Duration::minutes(ACCESS_TOKEN_DURATION)).timestamp(),
        }; 

        let refresh_claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            user_role: user.user_role,
            exp: (now + Duration::minutes(REFRESH_TOKEN_DURATION)).timestamp(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let jwt_secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
                &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }
} 
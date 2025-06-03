use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};
use rand::rngs::OsRng;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub user_role: UserRole,
    pub is_email_verified: bool,
    pub otp_code: Option<String>,
    pub otp_expires_at: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
    pub verification_token_expires_at: Option<DateTime<Utc>>,
    pub reset_token: Option<String>,
    pub reset_token_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

impl User {
    pub async fn create(pool: &PgPool, dto: CreateUserDto) -> Result<User, sqlx::Error> {
        let password_hash = Self::hash_password(&dto.password)?;
        let now = Utc::now();
        let verification_token = Uuid::new_v4().to_string();
        let verification_expires = now + chrono::Duration::hours(24);

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (
                username, email, password_hash, user_role, 
                verification_token, verification_token_expires_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, 'user', $4, $5, $6, $6)
            RETURNING 
                id, username, email, password_hash,
                user_role as "user_role: UserRole",
                is_email_verified,
                verification_token,
                verification_token_expires_at,
                reset_token,
                reset_token_expires_at,
                created_at,
                updated_at
            "#,
            dto.username,
            dto.email,
            password_hash,
            verification_token,
            verification_expires,
            now
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash,
                user_role as "user_role: UserRole",
                is_email_verified,
                verification_token,
                verification_token_expires_at,
                reset_token,
                reset_token_expires_at,
                created_at,
                updated_at
            FROM users 
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn verify_email(pool: &PgPool, token: &str) -> Result<bool, sqlx::Error> {
        let now = Utc::now();
        let result = sqlx::query!(
            r#"
            UPDATE users 
            SET is_email_verified = true,
                verification_token = NULL,
                verification_token_expires_at = NULL,
                updated_at = $1
            WHERE verification_token = $2 
            AND verification_token_expires_at > $1
            AND is_email_verified = false
            "#,
            now,
            token
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_password_reset(pool: &PgPool, email: &str) -> Result<Option<String>, sqlx::Error> {
        let user = Self::find_by_email(pool, email).await?;
        
        if let Some(_) = user {
            let reset_token = Uuid::new_v4().to_string();
            let now = Utc::now();
            let expires = now + chrono::Duration::hours(1);

            sqlx::query!(
                r#"
                UPDATE users 
                SET reset_token = $1,
                    reset_token_expires_at = $2,
                    updated_at = $3
                WHERE email = $4
                "#,
                reset_token,
                expires,
                now,
                email
            )
            .execute(pool)
            .await?;

            Ok(Some(reset_token))
        } else {
            Ok(None)
        }
    }

    pub async fn reset_password(
        pool: &PgPool,
        token: &str,
        new_password: &str
    ) -> Result<bool, sqlx::Error> {
        let now = Utc::now();
        let password_hash = Self::hash_password(new_password)?;

        let result = sqlx::query!(
            r#"
            UPDATE users 
            SET password_hash = $1,
                reset_token = NULL,
                reset_token_expires_at = NULL,
                updated_at = $2
            WHERE reset_token = $3 
            AND reset_token_expires_at > $2
            "#,
            password_hash,
            now,
            token
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(&self.password_hash)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    fn hash_password(password: &str) -> Result<String, sqlx::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2.hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| sqlx::Error::Protocol(format!("Password hashing failed: {}", e).into()))
    }

    pub fn is_otp_valid(&self, provided_otp: &str) -> bool {
        match (&self.otp_code, &self.otp_expires_at) {
            (Some(stored_otp), Some(expires_at)) => {
                stored_otp == provided_otp && Utc::now() < *expires_at
            }
            _ => false
        }
    }
} 
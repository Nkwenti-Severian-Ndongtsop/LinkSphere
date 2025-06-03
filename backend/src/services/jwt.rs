#![allow(dead_code)]

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, errors::Error as JwtError};
use serde::{Serialize, Deserialize};
use std::env;
use chrono::{Utc, Duration};
use crate::models::user::UserRole;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,  // user_id
    pub username: String,
    pub email: String,
    pub user_role: UserRole,
    pub exp: i64,  // expiration time
    pub iat: i64,  // issued at
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let secret = env::var("JWT_SECRET")?;
        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        })
    }

    pub fn generate_token(&self, user_id: i32, username: &str, email: &str, user_role: UserRole) -> Result<String, JwtError> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24); // Token expires in 24 hours

        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            email: email.to_string(),
            user_role,
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
} 
#![allow(dead_code)]

use serde::Serialize;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken;
use sqlx;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
    pub error_type: ErrorType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    ValidationError,
    DatabaseError,
    NotFound,
    Unauthorized,
    ServerError,
    TokenExpired,
    InvalidCredentials,
    EmailNotVerified,
    EmailAlreadyExists,
    UsernameAlreadyExists,
    AdminAlreadyExists,
    Forbidden,
    InvalidToken,
    Configuration,
    Internal,
    Validation,
}

impl AppError {
    pub fn new(message: impl Into<String>, error_type: ErrorType) -> Self {
        Self {
            message: message.into(),
            error_type,
            details: None,
        }
    }

    pub fn with_details(message: impl Into<String>, error_type: ErrorType, details: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_type,
            details: Some(details.into()),
        }
    }

    // Common error constructors
    pub fn invalid_credentials() -> Self {
        Self::new("Invalid credentials", ErrorType::InvalidCredentials)
    }

    pub fn email_not_verified() -> Self {
        Self::new("Email not verified", ErrorType::EmailNotVerified)
    }

    pub fn email_already_exists() -> Self {
        Self::new("Email already exists", ErrorType::EmailAlreadyExists)
    }

    pub fn invalid_token() -> Self {
        Self::new("Invalid token", ErrorType::InvalidToken)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self.error_type {
            ErrorType::ValidationError |
            ErrorType::Validation => StatusCode::BAD_REQUEST,
            ErrorType::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::NotFound => StatusCode::NOT_FOUND,
            ErrorType::Unauthorized | 
            ErrorType::InvalidCredentials |
            ErrorType::EmailNotVerified |
            ErrorType::TokenExpired |
            ErrorType::InvalidToken => StatusCode::UNAUTHORIZED,
            ErrorType::EmailAlreadyExists |
            ErrorType::UsernameAlreadyExists |
            ErrorType::AdminAlreadyExists => StatusCode::BAD_REQUEST,
            ErrorType::Forbidden => StatusCode::FORBIDDEN,
            ErrorType::ServerError |
            ErrorType::Configuration |
            ErrorType::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => AppError::new("Resource not found", ErrorType::NotFound),
            _ => AppError::with_details(
                "Database error occurred",
                ErrorType::DatabaseError,
                error.to_string()
            ),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AppError::new("Token has expired", ErrorType::TokenExpired)
            },
            _ => AppError::with_details(
                "Invalid token",
                ErrorType::InvalidToken,
                error.to_string()
            ),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.details {
            Some(details) => write!(f, "{}: {}", self.message, details),
            None => write!(f, "{}", self.message),
        }
    }
}
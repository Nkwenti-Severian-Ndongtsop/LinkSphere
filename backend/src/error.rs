#![allow(dead_code)]

use serde::Serialize;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken;
use sqlx;

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
    // Currently used error types
    ValidationError,
    DatabaseError,
    NotFound,
    Unauthorized,
    ServerError,
    TokenExpired,

    // Reserved for future use
    InvalidCredentials,
    
    EmailNotVerified,
    
    EmailAlreadyExists,
    
    UsernameAlreadyExists,
    
    AdminAlreadyExists,
    
    Forbidden,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self.error_type {
            ErrorType::ValidationError => StatusCode::BAD_REQUEST,
            ErrorType::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::NotFound => StatusCode::NOT_FOUND,
            ErrorType::Unauthorized | 
            ErrorType::InvalidCredentials |
            ErrorType::EmailNotVerified |
            ErrorType::TokenExpired => StatusCode::UNAUTHORIZED,
            ErrorType::EmailAlreadyExists |
            ErrorType::UsernameAlreadyExists |
            ErrorType::AdminAlreadyExists => StatusCode::BAD_REQUEST,
            ErrorType::Forbidden => StatusCode::FORBIDDEN,
            ErrorType::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => AppError {
                message: "Resource not found".to_string(),
                error_type: ErrorType::NotFound,
                details: None,
            },
            _ => AppError {
                message: "Database error occurred".to_string(),
                error_type: ErrorType::DatabaseError,
                details: Some(error.to_string()),
            },
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError {
                message: "Token has expired".to_string(),
                error_type: ErrorType::TokenExpired,
                details: None,
            },
            _ => AppError {
                message: "Invalid token".to_string(),
                error_type: ErrorType::Unauthorized,
                details: Some(error.to_string()),
            },
        }
    }
}
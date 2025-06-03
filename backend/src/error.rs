use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use jsonwebtoken::errors::Error as JwtError;
use serde::Serialize;
use sqlx::Error as SqlxError;

#[derive(Debug, Display)]
pub enum AppError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "Invalid Credentials")]
    InvalidCredentials,

    #[display(fmt = "Email Not Verified")]
    EmailNotVerified,

    #[display(fmt = "Invalid Token")]
    InvalidToken,

    #[display(fmt = "Token Expired")]
    TokenExpired,

    #[display(fmt = "Email Already Exists")]
    EmailAlreadyExists,

    #[display(fmt = "Username Already Exists")]
    UsernameAlreadyExists,

    #[display(fmt = "Admin Already Exists")]
    AdminAlreadyExists,

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "Forbidden")]
    Forbidden,

    #[display(fmt = "Database Error: {}", _0)]
    DatabaseError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InternalServerError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    code: "internal_server_error".into(),
                    message: self.to_string(),
                })
            }
            AppError::InvalidCredentials => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    code: "invalid_credentials".into(),
                    message: self.to_string(),
                })
            }
            AppError::EmailNotVerified => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    code: "email_not_verified".into(),
                    message: self.to_string(),
                })
            }
            AppError::InvalidToken => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    code: "invalid_token".into(),
                    message: self.to_string(),
                })
            }
            AppError::TokenExpired => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    code: "expired_token".into(),
                    message: self.to_string(),
                })
            }
            AppError::EmailAlreadyExists => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    code: "email_in_use".into(),
                    message: self.to_string(),
                })
            }
            AppError::UsernameAlreadyExists => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    code: "username_in_use".into(),
                    message: self.to_string(),
                })
            }
            AppError::AdminAlreadyExists => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    code: "admin_exists".into(),
                    message: self.to_string(),
                })
            }
            AppError::Unauthorized => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    code: "unauthorized".into(),
                    message: self.to_string(),
                })
            }
            AppError::Forbidden => {
                HttpResponse::Forbidden().json(ErrorResponse {
                    code: "forbidden".into(),
                    message: self.to_string(),
                })
            }
            AppError::DatabaseError(msg) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    code: "database_error".into(),
                    message: msg.clone(),
                })
            }
        }
    }
}

impl From<SqlxError> for AppError {
    fn from(error: SqlxError) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}

impl From<JwtError> for AppError {
    fn from(error: JwtError) -> Self {
        match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        }
    }
} 
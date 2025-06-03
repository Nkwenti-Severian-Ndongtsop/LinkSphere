use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::services::auth::AuthService;
use crate::models::user::{CreateUserDto, LoginDto};
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct EmailVerificationRequest {
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    token: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse<T> {
    success: bool,
    data: T,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/verify-email", web::post().to(verify_email))
            .route("/request-password-reset", web::post().to(request_password_reset))
            .route("/reset-password", web::post().to(reset_password))
            .route("/refresh", web::post().to(refresh_token)),
    );
}

async fn register(
    auth_service: web::Data<AuthService>,
    credentials: web::Json<CreateUserDto>,
) -> Result<impl Responder, AppError> {
    let (user, tokens) = auth_service.register(credentials.into_inner()).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        data: tokens,
    }))
}

async fn login(
    auth_service: web::Data<AuthService>,
    credentials: web::Json<LoginDto>,
) -> Result<impl Responder, AppError> {
    let (user, tokens) = auth_service.login(credentials.into_inner()).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        data: tokens,
    }))
}

async fn verify_email(
    auth_service: web::Data<AuthService>,
    verification: web::Json<EmailVerificationRequest>,
) -> Result<impl Responder, AppError> {
    auth_service.verify_email(&verification.token).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        data: "Email verified successfully",
    }))
}

async fn request_password_reset(
    auth_service: web::Data<AuthService>,
    request: web::Json<PasswordResetRequest>,
) -> Result<impl Responder, AppError> {
    auth_service.create_password_reset(&request.email).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        data: "Password reset email sent",
    }))
}

async fn reset_password(
    auth_service: web::Data<AuthService>,
    request: web::Json<ResetPasswordRequest>,
) -> Result<impl Responder, AppError> {
    auth_service
        .reset_password(&request.token, &request.new_password)
        .await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        data: "Password reset successfully",
    }))
}

async fn refresh_token(
    auth_service: web::Data<AuthService>,
    request: web::Json<RefreshTokenRequest>,
) -> Result<impl Responder, AppError> {
    let tokens = auth_service.refresh_token(&request.refresh_token).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        data: tokens,
    }))
} 
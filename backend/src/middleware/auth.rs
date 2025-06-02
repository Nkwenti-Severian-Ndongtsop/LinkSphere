use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
    body::Body,
};
use axum_extra::headers::{authorization::Bearer, Authorization, HeaderMapExt};
use crate::{
    database::state::AppState, routes::auth::{AuthError, AuthErrorType}, services::JwtService
};

pub async fn auth_middleware(
    State(_): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthError>)> {
    // Get the token from the Authorization header
    let auth_header = req
        .headers()
        .typed_get::<Authorization<Bearer>>()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(AuthError {
                    message: "Missing authorization header".to_string(),
                    error_type: AuthErrorType::MissingToken,
                    details: None,
                }),
            )
        })?;

    let jwt_service = JwtService::new().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthError {
                message: "JWT service error".to_string(),
                error_type: AuthErrorType::ServerError,
                details: None,
            }),
        )
    })?;

    // Verify the token
    let claims = jwt_service.verify_token(auth_header.0.token()).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                message: "Invalid token".to_string(),
                error_type: AuthErrorType::InvalidToken,
                details: None,
            }),
        )
    })?;

    // Add the user claims to the request extensions
    req.extensions_mut().insert(claims);

    // Continue with the request
    Ok(next.run(req).await)
} 
use std::sync::Arc;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
    body::Body,
};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::routes::auth::{AuthError, AuthErrorType};

#[derive(Clone)]
pub struct RateLimiter {
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_attempts: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: usize, window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window: Duration::from_secs(window_secs),
        }
    }

    async fn is_rate_limited(&self, key: &str) -> bool {
        let mut attempts = self.attempts.lock().await;
        let now = Instant::now();
        
        // Clean up old attempts
        if let Some(timestamps) = attempts.get_mut(key) {
            timestamps.retain(|&timestamp| now.duration_since(timestamp) < self.window);
            
            if timestamps.len() >= self.max_attempts {
                return true;
            }
            
            timestamps.push(now);
        } else {
            attempts.insert(key.to_string(), vec![now]);
        }
        
        false
    }
}

pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<AuthError>)> {
    // Use IP address as rate limit key (you might want to use a different strategy)
    let key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    if limiter.is_rate_limited(&key).await {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(AuthError {
                message: "Too many requests".to_string(),
                error_type: AuthErrorType::ServerError,
                details: None,
            }),
        ));
    }

    Ok(next.run(req).await)
} 
// Declare modules
mod database;
mod models;
mod routes;
mod services;
mod middleware;
mod error;

use axum::{
    http::Method,
    routing::{delete, get, post},
    Router,
    middleware::{from_fn_with_state},
};
use std::net::SocketAddr;
use dotenvy::dotenv;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use std::env;

// Import necessary items from modules
use database::state::{PoolState, AppState};
use database::db::{create_pool, run_migrations};
use routes::{
    root_handler, 
    get_links_handler, 
    delete_link_handler, 
    increment_click_count_handler,
    create_link_handler,
    create_user_handler,
    auth::{login_handler, register_handler, logout_handler, verify_email_handler},
};
use middleware::{
    auth::auth_middleware,
    rate_limit::{RateLimiter, rate_limit_middleware},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("🔧 Loading environment variables...");

    // Create database pool using db module
    let pool = match create_pool().await {
        Ok(pool) => {
            println!("✅ Successfully connected to database!");
            pool
        }
        Err(err) => {
            eprintln!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    // Run migrations using db module
    println!("🔄 Running database migrations...");
    if let Err(err) = run_migrations(&pool).await {
        eprintln!("🔥 Migration error: {:?}", err);
        std::process::exit(1); // Exit on migration error as we need the schema
    }
    println!("✅ Database migrations completed successfully!");

    // Create AppState
    let app_state: AppState = Arc::new(PoolState { pool });

    // Initialize rate limiter
    let max_attempts = std::env::var("RATE_LIMIT_MAX_ATTEMPTS")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);
    let window_secs = std::env::var("RATE_LIMIT_WINDOW_SECS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);
    let rate_limiter = RateLimiter::new(max_attempts, window_secs);

    // Define CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    // Build the router with public routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/register", post(register_handler))
        .route("/api/auth/verify-email", post(verify_email_handler))
        .route("/api/auth/logout", post(logout_handler))
        .route("/api/users", post(create_user_handler))
        .layer(cors)
        .layer(from_fn_with_state(rate_limiter, rate_limit_middleware))
        .with_state(app_state.clone());

    // Protected routes with middleware
    let protected_routes = Router::new()
        .route("/api/links", get(get_links_handler))
        .route("/api/links", post(create_link_handler))
        .route("/api/links/:id", delete(delete_link_handler))
        .route("/api/links/:id/click", post(increment_click_count_handler))
        .layer(from_fn_with_state(app_state.clone(), auth_middleware))
        .with_state(app_state);

    let app = app.merge(protected_routes);

    // Get port from environment variable or use 3000 as default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    // Get host from environment variable or use 0.0.0.0 as default
    let host = env::var("HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        .parse::<std::net::IpAddr>()
        .expect("HOST must be a valid IP address");

    // Server startup logic
    let addr = SocketAddr::from((host, port));
    println!("🚀 Server starting on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("🔥 Failed to bind server address: {:?}", err);
            std::process::exit(1);
        }
    };

    println!("✨ Server is ready to accept connections");
    if let Err(err) = axum::serve(listener, app.into_make_service()).await {
        eprintln!("🔥 Server error: {:?}", err);
    }

    Ok(())
}

// --- Handlers and Structs moved to respective modules ---

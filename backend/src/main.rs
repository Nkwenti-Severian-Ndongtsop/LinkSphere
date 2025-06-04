// Declare modules
mod database;
mod models;
mod routes;
mod services;
mod middleware;
mod error;
mod logging;

use axum::{
    http::Method,
    routing::{delete, get, post, put},
    Router,
    middleware::{from_fn_with_state},
};
use sqlx::PgPool;
use std::net::SocketAddr;
use dotenvy::dotenv;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_cookies::CookieManagerLayer;
use std::env;
use tracing::{info, error};

// Import necessary items from modules
use database::state::{PoolState, AppState};
use database::db::{create_pool, run_migrations};
use routes::{
    root_handler, 
    get_links_handler, 
    delete_link_handler, 
    increment_click_count_handler,
    create_link_handler,
    update_link_handler,
    get_link_tags_handler,
    add_link_tag_handler,
    remove_link_tag_handler,
    create_user_handler,
    auth::{login_handler, register_handler, logout_handler, verify_email_handler},
    get_user_stats_handler,
    get_all_links_handler,
    delete_user_handler,
    delete_any_link_handler,
    tags::{get_tags_handler, create_tag_handler, delete_tag_handler},
    categories::{get_categories_handler, create_category_handler, delete_category_handler},
    profile::{get_profile_handler, update_profile_handler, change_password_handler},
};
use middleware::{
    auth::auth_middleware,
    rate_limit::{RateLimiter, rate_limit_middleware},
};

async fn create_default_admin(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let admin_email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@linksphere.com".to_string());
    let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "Admin@123".to_string());
    let admin_username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());

    // Check if admin already exists
    let admin_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE user_role = 'admin'::user_role) as exists",
    )
    .fetch_one(pool)
    .await?
    .exists
    .unwrap_or(false);

    if !admin_exists {
        info!("Creating default admin account...");
        let password_hash = routes::auth::hash_password(&admin_password).unwrap();

        sqlx::query!(
            r#"
            INSERT INTO users (
                username, email, password_hash, user_role, 
                is_email_verified, created_at, updated_at
            )
            VALUES ($1, $2, $3, 'admin'::user_role, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
            admin_username,
            admin_email,
            password_hash,
        )
        .execute(pool)
        .await?;

        info!("✅ Default admin account created successfully");
    } else {
        info!("✅ Admin account already exists");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Initialize logging first
    if let Err(e) = logging::setup_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }
    
    info!("🚀 Starting LinkSphere backend server...");
    info!("🔧 Loading environment variables...");

    // Create database pool using db module
    let pool = match create_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to create database pool: {}", e);
            std::process::exit(1);
        }
    };

    // Run database migrations
    if let Err(e) = run_migrations(&pool).await {
        error!("Failed to run database migrations: {}", e);
        std::process::exit(1);
    }

    // Create rate limiter
    let rate_limiter = Arc::new(RateLimiter::new(10, 60));

    // Create app state
    let app_state: AppState = Arc::new(PoolState { pool: pool.clone() });

    // Create default admin account
    if let Err(e) = create_default_admin(&pool).await {
        error!("Failed to create admin account: {}", e);
        std::process::exit(1);
    }

    // Start the cleanup service
    info!("🧹 Starting cleanup service...");
    let cleanup_service = services::CleanupService::new(pool.clone());
    cleanup_service.start_cleanup_task().await;
    info!("✅ Cleanup service started successfully!");

    // Define CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);
    info!("🔒 CORS configuration applied");

    // Build the router with public routes
    info!("🛠️ Configuring routes...");
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/register", post(register_handler))
        .route("/api/auth/verify-email", post(verify_email_handler))
        .route("/api/auth/logout", post(logout_handler))
        .route("/api/users", post(create_user_handler))
        .layer(cors)
        .layer(CookieManagerLayer::new())
        .layer(from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(logging::create_trace_layer())
        .with_state(app_state.clone());

    // Protected user routes
    let protected_routes = Router::new()
        .route("/api/links", get(get_links_handler))
        .route("/api/links", post(create_link_handler))
        .route("/api/links/:id", delete(delete_link_handler))
        .route("/api/links/:id", put(update_link_handler))
        .route("/api/links/:id/click", post(increment_click_count_handler))
        .route("/api/links/:id/tags", get(get_link_tags_handler))
        .route("/api/links/:id/tags/:tag_id", post(add_link_tag_handler))
        .route("/api/links/:id/tags/:tag_id", delete(remove_link_tag_handler))
        .route("/api/tags", get(get_tags_handler))
        .route("/api/tags", post(create_tag_handler))
        .route("/api/tags/:id", delete(delete_tag_handler))
        .route("/api/categories", get(get_categories_handler))
        .route("/api/categories", post(create_category_handler))
        .route("/api/categories/:id", delete(delete_category_handler))
        .route("/api/users/me", get(get_profile_handler))
        .route("/api/users/me", put(update_profile_handler))
        .route("/api/users/me/password", put(change_password_handler))
        .layer(from_fn_with_state(app_state.clone(), auth_middleware))
        .with_state(app_state.clone());

    // Admin routes
    let admin_routes = Router::new()
        .route("/api/admin/users", get(get_user_stats_handler))
        .route("/api/admin/links", get(get_all_links_handler))
        .route("/api/admin/users/:id", delete(delete_user_handler))
        .route("/api/admin/links/:id", delete(delete_any_link_handler))
        .layer(from_fn_with_state(app_state.clone(), auth_middleware))
        .with_state(app_state);

    let app = app
        .merge(protected_routes)
        .merge(admin_routes);

    info!("✅ Routes configured successfully!");

    // Get port from environment variable or use 3000 as default
    let port = match env::var("PORT") {
        Ok(port_str) => match port_str.parse::<u16>() {
            Ok(port_num) => {
                info!("Using configured port: {}", port_num);
                port_num
            },
            Err(e) => {
                error!("Invalid PORT value '{}': {}. Using default port 3000", port_str, e);
                3000
            }
        },
        Err(e) => {
            info!("PORT not set ({}). Using default port 3000", e);
            3000
        }
    };

    // Get host from environment variable or use 0.0.0.0 as default
    let host = match env::var("HOST") {
        Ok(host_str) => match host_str.parse::<std::net::IpAddr>() {
            Ok(host_addr) => {
                info!("Using configured host: {}", host_addr);
                host_addr
            },
            Err(e) => {
                error!("Invalid HOST value '{}': {}. Using default host 0.0.0.0", host_str, e);
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))
            }
        },
        Err(e) => {
            info!("HOST not set ({}). Using default host 0.0.0.0", e);
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))
        }
    };

    // Server startup logic
    let addr = SocketAddr::from((host, port));
    info!("🚀 Server starting on {}", addr);

    info!("✨ Server is ready to accept connections");
    match axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await 
    {
        Ok(_) => {
            info!("👋 Server shutdown gracefully");
            Ok(())
        },
        Err(err) => {
            error!("🔥 Server failed to start: {}", err);
            Err(err.into())
        }
    }
}

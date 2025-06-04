use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::{env, time::Duration};
use tracing::{info, warn, error};

const MAX_CONNECTIONS: u32 = 5;
const CONNECTION_TIMEOUT: u64 = 30;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: u64 = 5;

#[derive(Debug)]
pub enum DatabaseError {
    Configuration(String),
    Connection(sqlx::Error),
    Migration(sqlx::migrate::MigrateError),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration(msg) => write!(f, "Database configuration error: {}", msg),
            Self::Connection(err) => write!(f, "Database connection error: {}", err),
            Self::Migration(err) => write!(f, "Database migration error: {}", err),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        DatabaseError::Connection(err)
    }
}

impl From<sqlx::migrate::MigrateError> for DatabaseError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        DatabaseError::Migration(err)
    }
}

// Function to create the database pool
pub async fn create_pool() -> Result<PgPool, DatabaseError> {
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| DatabaseError::Configuration("DATABASE_URL must be set".to_string()))?;

    info!("🔌 Attempting to connect to database...");

    let mut retries = 0;
    loop {
        match PgPoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .acquire_timeout(Duration::from_secs(CONNECTION_TIMEOUT))
            .connect(&database_url)
            .await {
                Ok(pool) => {
                    info!("✅ Successfully connected to database");
                    return Ok(pool);
                }
                Err(e) => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        error!("❌ Failed to connect to database after {} attempts", MAX_RETRIES);
                        return Err(DatabaseError::Connection(e));
                    }
                    warn!("⚠️ Failed to connect to database (attempt {}/{}), retrying in {} seconds...", 
                        retries, MAX_RETRIES, RETRY_DELAY);
                    tokio::time::sleep(Duration::from_secs(RETRY_DELAY)).await;
                }
            }
    }
}

// Function to run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<(), DatabaseError> {
    info!("🔄 Running database migrations...");
    
    match sqlx::migrate!("./migrations").run(pool).await {
        Ok(_) => {
            info!("✅ Database migrations completed successfully");
            Ok(())
        }
        Err(err) => {
            error!("❌ Failed to run database migrations: {:?}", err);
            Err(DatabaseError::Migration(err))
        }
    }
} 
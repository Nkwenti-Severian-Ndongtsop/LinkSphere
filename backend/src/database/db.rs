use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use sqlx::PgPool;
use std::env;

// Function to create the database pool
pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    // We don't need to store the database_url since we're using direct configuration
    env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    println!("Database URL loaded");

    // Create connection options with SSL required
    let options = PgConnectOptions::new()
        .host("dpg-d0tb7de3jp1c73edeihg-a.frankfurt-postgres.render.com")
        .port(5432)
        .username("cdn")
        .password("xHv8Y4unSXJ1n0umHCDXrpkuO2gMDEUb")
        .database("links_db")
        .ssl_mode(sqlx::postgres::PgSslMode::Require);

    PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

// Function to run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
     match sqlx::migrate!("./migrations").run(pool).await {
         Ok(_) => {
            println!("✅ Database migrations ran successfully!");
            Ok(())
         },
         Err(err) => {
             eprintln!("🔥 Failed to run database migrations: {:?}", err);
             // Consider exiting or more robust error handling in production
             Err(err)
         }
    }
} 
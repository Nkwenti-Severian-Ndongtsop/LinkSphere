use sqlx::PgPool;
use chrono::Utc;
use std::time::Duration;
use tokio::time;
use tracing::{info, error};

pub struct CleanupService {
    pool: PgPool,
}

impl CleanupService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn start_cleanup_task(self) {
        info!("🧹 Starting cleanup task with 5-minute interval");
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(300)); // Run every 5 minutes
            loop {
                interval.tick().await;
                info!("🔄 Running cleanup check...");
                if let Err(e) = self.cleanup_expired_unverified_users().await {
                    error!("❌ Error during cleanup: {}", e);
                }
            }
        });
    }

    async fn cleanup_expired_unverified_users(&self) -> Result<(), sqlx::Error> {
        // Delete unverified users whose OTP has expired
        let now = Utc::now();
        
        let result = sqlx::query!(
            r#"
            DELETE FROM users 
            WHERE is_email_verified = false 
            AND otp_expires_at < $1
            RETURNING id, email
            "#,
            now
        )
        .fetch_all(&self.pool)
        .await?;

        // Log the cleanup results
        let count = result.len();
        if count > 0 {
            info!("🗑️ Cleaned up {} expired unverified accounts:", count);
            for row in result {
                info!("  • Removed user: {} (ID: {})", row.email, row.id);
            }
        } else {
            info!("✨ No expired unverified accounts to clean up");
        }

        Ok(())
    }
} 
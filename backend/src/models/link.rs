#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: String,
    pub description: String,
    pub uploader_name: String,
    pub click_count: i32,
    pub favicon_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLinkRequest {
    pub url: String,
    pub title: String,
    pub description: String,
    pub favicon_url: Option<String>,
}

impl Link {
    pub async fn create(pool: &PgPool, user_id: Uuid, req: CreateLinkRequest) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            INSERT INTO links (user_id, url, title, description, click_count, favicon_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 0, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING *
            "#,
            user_id,
            req.url,
            req.title,
            req.description,
            req.favicon_url
        )
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT 
                id, user_id, url, title, description, uploader_name,
                click_count, favicon_url, created_at, updated_at
            FROM links 
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT 
                id, user_id, url, title, description, uploader_name,
                click_count, favicon_url, created_at, updated_at
            FROM links 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM links WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
} 
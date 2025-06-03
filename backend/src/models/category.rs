use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
}

impl Category {
    pub async fn create(
        pool: &PgPool,
        name: String,
        description: Option<String>,
        user_id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            INSERT INTO categories (name, description, user_id, created_at, updated_at)
            VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING *
            "#,
            name,
            description,
            user_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM categories
            WHERE user_id = $1
            ORDER BY name
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_link(pool: &PgPool, link_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT c.* FROM categories c
            INNER JOIN link_categories lc ON lc.category_id = c.id
            WHERE lc.link_id = $1
            ORDER BY c.name
            "#,
            link_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: String,
        description: Option<String>,
        user_id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            UPDATE categories
            SET name = $1,
                description = $2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $3 AND user_id = $4
            RETURNING *
            "#,
            name,
            description,
            id,
            user_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM categories WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
} 
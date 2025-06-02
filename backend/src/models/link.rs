use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Represents a link in the system
#[derive(Serialize, FromRow, Debug, Clone)]
pub struct Link {
    #[sqlx(rename = "id")]
    pub id: i32,
    #[sqlx(rename = "user_id")]
    pub user_id: i32,
    #[sqlx(rename = "url")]
    pub url: String,
    #[sqlx(rename = "title")]
    pub title: String,
    #[sqlx(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[sqlx(rename = "click_count")]
    pub click_count: i32,
    #[sqlx(rename = "favicon_url")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon_url: Option<String>,
    #[sqlx(rename = "created_at")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "uploader_username")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploader_username: Option<String>,
}

/// Request payload for creating a new link
#[derive(Deserialize, Debug)]
pub struct CreateLinkRequest {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub user_id: i32,  // For now, we'll require user_id. In a real app, this would come from auth
} 
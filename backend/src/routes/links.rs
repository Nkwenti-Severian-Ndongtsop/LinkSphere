use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{postgres::PgRow, FromRow, Row};
use crate::{
    database::state::AppState,
    error::{AppError, ErrorType},
    routes::auth::AuthUser,
};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: String,
    pub description: String,
    pub click_count: i32,
    pub favicon_url: Option<String>,
    pub uploader_name: String,
    pub tags: Option<Vec<Tag>>,
    pub categories: Option<Vec<Category>>,
}

// Implement FromRow for Link to handle custom deserialization
impl<'r> FromRow<'r, PgRow> for Link {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Link {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            url: row.try_get("url")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            click_count: row.try_get("click_count")?,
            favicon_url: row.try_get("favicon_url")?,
            uploader_name: row.try_get("uploader_name")?,
            tags: None,
            categories: None,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub url: String,
    pub title: String,
    pub description: String,
    pub favicon_url: Option<String>,
    pub tags: Option<Vec<Uuid>>,
    pub categories: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub favicon_url: Option<String>,
}

// Create a new link
pub async fn create_link_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateLinkRequest>,
) -> Result<Json<Link>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();
    
    // Start transaction
    let mut tx = state.pool.begin().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                "Failed to start transaction",
                ErrorType::DatabaseError,
            )),
        )
    })?;

    // Create link
    let mut link = sqlx::query_as::<_, Link>(
        r#"
        INSERT INTO links (user_id, url, title, description, favicon_url, click_count, uploader_name)
        VALUES ($1, $2, $3, $4, $5, 0, (SELECT username FROM users WHERE id = $1))
        RETURNING id, user_id, url, title, description, click_count, favicon_url, uploader_name
        "#
    )
    .bind(user_id)
    .bind(&payload.url)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.favicon_url)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to create link: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    // Add tags if provided
    if let Some(tags) = payload.tags {
        for tag_id in tags {
            sqlx::query(
                r#"
                INSERT INTO link_tags (link_id, tag_id)
                VALUES ($1, $2)
                "#
            )
            .bind(link.id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AppError::new(
                        &format!("Failed to add tags: {}", e),
                        ErrorType::DatabaseError,
                    )),
                )
            })?;
        }
    }

    // Add categories if provided
    if let Some(categories) = payload.categories {
        for category_id in categories {
            sqlx::query(
                r#"
                INSERT INTO link_categories (link_id, category_id)
                VALUES ($1, $2)
                "#
            )
            .bind(link.id)
            .bind(category_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AppError::new(
                        &format!("Failed to add categories: {}", e),
                        ErrorType::DatabaseError,
                    )),
                )
            })?;
        }
    }

    // Commit transaction
    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to commit transaction: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    // Initialize empty tags and categories
    link.tags = Some(Vec::new());
    link.categories = Some(Vec::new());

    Ok(Json(link))
}

// Get all links for current user
pub async fn get_links_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Link>>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();
    
    let mut links = sqlx::query_as::<_, Link>(
        r#"
        SELECT 
            l.id,
            l.user_id,
            l.url,
            l.title,
            l.description,
            l.click_count,
            l.favicon_url,
            u.username as uploader_name
        FROM links l
        JOIN users u ON l.user_id = u.id
        WHERE l.user_id = $1
        ORDER BY l.created_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,  
            Json(AppError::new(
                &format!("Failed to fetch links: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    // Fetch tags and categories for each link
    for link in &mut links {
        // Get tags
        link.tags = Some(
            sqlx::query_as::<_, Tag>(
                r#"
                SELECT t.id, t.name, t.user_id
                FROM tags t
                JOIN link_tags lt ON t.id = lt.tag_id
                WHERE lt.link_id = $1
                "#
            )
            .bind(link.id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AppError::new(
                        &format!("Failed to fetch tags: {}", e),
                        ErrorType::DatabaseError,
                    )),
                )
            })?
        );

        // Get categories
        link.categories = Some(
            sqlx::query_as::<_, Category>(
                r#"
                SELECT c.id, c.name, c.description, c.user_id
                FROM categories c
                JOIN link_categories lc ON c.id = lc.category_id
                WHERE lc.link_id = $1
                "#
            )
            .bind(link.id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AppError::new(
                        &format!("Failed to fetch categories: {}", e),
                        ErrorType::DatabaseError,
                    )),
                )
            })?
        );
    }

    Ok(Json(links))
}

// Update a link
pub async fn update_link_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(link_id): Path<Uuid>,
    Json(payload): Json<UpdateLinkRequest>,
) -> Result<Json<Link>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    let link = sqlx::query_as::<_, Link>(
        r#"
        UPDATE links
        SET 
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            url = COALESCE($3, url),
            favicon_url = $4
        WHERE id = $5 AND user_id = $6
        RETURNING id, user_id, url, title, description, click_count, favicon_url, uploader_name
        "#
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.url)
    .bind(payload.favicon_url)
    .bind(link_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to update link: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    match link {
        Some(mut link) => {
            // Fetch tags and categories for the updated link
            link.tags = Some(
                sqlx::query_as::<_, Tag>(
                    r#"
                    SELECT t.id, t.name, t.user_id
                    FROM tags t
                    JOIN link_tags lt ON t.id = lt.tag_id
                    WHERE lt.link_id = $1
                    "#
                )
                .bind(link.id)
                .fetch_all(&state.pool)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(AppError::new(
                            &format!("Failed to fetch tags: {}", e),
                            ErrorType::DatabaseError,
                        )),
                    )
                })?
            );

            link.categories = Some(
                sqlx::query_as::<_, Category>(
                    r#"
                    SELECT c.id, c.name, c.description, c.user_id
                    FROM categories c
                    JOIN link_categories lc ON c.id = lc.category_id
                    WHERE lc.link_id = $1
                    "#
                )
                .bind(link.id)
                .fetch_all(&state.pool)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(AppError::new(
                            &format!("Failed to fetch categories: {}", e),
                            ErrorType::DatabaseError,
                        )),
                    )
                })?
            );

            Ok(Json(link))
        },
        None => Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        )),
    }
}

// Delete a link
pub async fn delete_link_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(link_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    let result = sqlx::query(
        r#"
        DELETE FROM links
        WHERE id = $1 AND user_id = $2
        "#
    )
    .bind(link_id)
    .bind(user_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to delete link: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Increment click count
pub async fn increment_click_count_handler(
    State(state): State<AppState>,
    Path(link_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let result = sqlx::query(
        r#"
        UPDATE links
        SET click_count = click_count + 1
        WHERE id = $1
        "#
    )
    .bind(link_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to increment click count: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Get link tags
pub async fn get_link_tags_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(link_id): Path<Uuid>,
) -> Result<Json<Vec<Tag>>, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.id, t.name, t.user_id
        FROM tags t
        JOIN link_tags lt ON t.id = lt.tag_id
        JOIN links l ON lt.link_id = l.id
        WHERE l.id = $1 AND l.user_id = $2
        "#
    )
    .bind(link_id)
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to fetch link tags: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(Json(tags))
}

// Add tag to link
pub async fn add_link_tag_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((link_id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    // Verify link ownership
    let link = sqlx::query(
        "SELECT id FROM links WHERE id = $1 AND user_id = $2"
    )
    .bind(link_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Database error: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if link.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        ));
    }

    // Add tag to link
    sqlx::query(
        r#"
        INSERT INTO link_tags (link_id, tag_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(link_id)
    .bind(tag_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to add tag to link: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// Remove tag from link
pub async fn remove_link_tag_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((link_id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<AppError>)> {
    let user_id = Uuid::parse_str(&auth_user.user_id).unwrap();

    // Verify link ownership
    let link = sqlx::query(
        "SELECT id FROM links WHERE id = $1 AND user_id = $2"
    )
    .bind(link_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Database error: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if link.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Link not found",
                ErrorType::NotFound,
            )),
        ));
    }

    // Remove tag from link
    let result = sqlx::query(
        r#"
        DELETE FROM link_tags
        WHERE link_id = $1 AND tag_id = $2
        "#
    )
    .bind(link_id)
    .bind(tag_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppError::new(
                &format!("Failed to remove tag from link: {}", e),
                ErrorType::DatabaseError,
            )),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(AppError::new(
                "Tag not found on link",
                ErrorType::NotFound,
            )),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
} 
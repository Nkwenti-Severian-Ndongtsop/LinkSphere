use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::database::state::AppState;
use crate::models::link::{Link, CreateLinkRequest};

// Handler to create a new link
pub async fn create_link_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateLinkRequest>,
) -> Result<Json<Link>, StatusCode> {
    let query = r#"
        INSERT INTO links (user_id, url, title, description, favicon_url, click_count)
        VALUES ($1, $2, $3, $4, $5, 0)
        RETURNING id, user_id, url, title, description, click_count, favicon_url, created_at, 
                 (SELECT username FROM users WHERE id = $1) as uploader_username
    "#;
    match sqlx::query_as::<_, Link>(query)
        .bind(payload.user_id)
        .bind(payload.url)
        .bind(payload.title)
        .bind(payload.description)
        .bind(payload.favicon_url)
        .fetch_one(&state.pool)
        .await
    {
        Ok(link) => {
            println!("Created new link with ID: {}", link.id);
            Ok(Json(link))
        }
        Err(e) => {
            eprintln!("Error creating link: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Handler to GET all links
pub async fn get_links_handler(
    State(state): State<AppState>
) -> Result<Json<Vec<Link>>, StatusCode> {
    let query = r#"
        SELECT
            l.id,
            l.user_id,
            l.url,
            l.title,
            l.description,
            l.click_count,
            l.favicon_url,
            l.created_at,
            u.username as uploader_username
        FROM links l
        LEFT JOIN users u ON l.user_id = u.id
        ORDER BY l.created_at DESC
    "#;

    match sqlx::query_as::<_, Link>(query)
        .fetch_all(&state.pool)
        .await
    {
        Ok(links) => Ok(Json(links)),
        Err(e) => {
            eprintln!("Error fetching links with usernames: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Handler to increment click count for a link
pub async fn increment_click_count_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let query = "UPDATE links SET click_count = click_count + 1 WHERE id = $1";

    match sqlx::query(query)
        .bind(id)
        .execute(&state.pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 1 {
                println!("Incremented click count for link ID: {}", id);
                Ok(StatusCode::NO_CONTENT)
            } else {
                println!("Link not found for click increment, ID: {}", id);
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            eprintln!("Error incrementing click count: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Handler to DELETE a link by ID
pub async fn delete_link_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let query = "DELETE FROM links WHERE id = $1";

    match sqlx::query(query)
        .bind(id)
        .execute(&state.pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 1 {
                println!("Deleted link with ID: {}", id);
                Ok(StatusCode::NO_CONTENT)
            } else {
                println!("Link not found with ID: {}", id);
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            eprintln!("Error deleting link: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
} 
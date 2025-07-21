use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::database::queries::{create_link, increment_click_count, get_link_by_id, update_link};
use crate::{
    api::{models::CreateLinkRequest, ApiResponse, ErrorResponse},
    database::{self, models::Link, PgPool},
    middleware::auth::AuthUser,
    services::link_preview::fetch_link_preview,
};
use uuid::Uuid;
use validator::Validate;

type LinkResponse = ApiResponse<Link>;
type LinksResponse = ApiResponse<Vec<Link>>;
/// Get all links
///
/// Returns a list of all links in the system
/// Requires Authentication: Bearer token from /api/auth/login
#[utoipa::path(
    get,
    path = "/api/links",
    responses(
        (status = 200, description = "Links retrieved successfully", body = LinksResponse),
        (status = 401, description = "Missing or invalid JWT token", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "links"
)]
pub async fn get_links(State(pool): State<PgPool>) -> impl IntoResponse {
    match database::get_all_links(&pool).await {
        Ok(links) => {
            let response = ApiResponse::success(links);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse::new(format!("Failed to fetch links: {e}"))
                .with_code("LINKS_FETCH_ERROR");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Create a new link
///
/// Creates a new link with the provided details. The user ID is automatically extracted from the JWT token.
/// Requires Authentication: Bearer token from /api/auth/login
///
#[utoipa::path(
    post,
    path = "/api/links",
    request_body = CreateLinkRequest,
    responses(
        (status = 201, description = "Link created successfully", body = LinkResponse),
        (status = 422, description = "Invalid request data (URL format, title/description length)", body = ErrorResponse),
        (status = 401, description = "Missing or invalid JWT token", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "links"
)]
pub async fn handle_create_link(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<CreateLinkRequest>,
) -> impl IntoResponse {
    // Validate the request payload
    if let Err(validation_errors) = payload.validate() {
        let error = ErrorResponse::new(format!("Validation error: {validation_errors}"))
            .with_code("VALIDATION_ERROR");
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(error)).into_response();
    }

    // Validate URL format
    if let Err(url_error) = payload.validate_url() {
        let error =
            ErrorResponse::new(format!("Invalid URL format: {url_error}")).with_code("INVALID_URL");
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(error)).into_response();
    }

    // Create the link first without preview
    let link = match create_link(
        &pool,
        payload.url.clone(),
        payload.title,
        payload.description,
        user.id,
        None, // No preview initially
    )
    .await
    {
        Ok(link) => link,
        Err(e) => {
            let error = ErrorResponse::new(format!("Failed to create link: {e}"))
                .with_code("LINK_CREATE_ERROR");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response();
        }
    };

    // Spawn a task to fetch and update the preview asynchronously
    let pool_clone = pool.clone();
    let url = payload.url.clone();
    let link_id = link.id;

    tokio::spawn(async move {
        if let Ok(preview) = fetch_link_preview(&url).await {
            // Update the link with the preview
            let _ = sqlx::query!(
                r#"
                UPDATE links 
                SET preview = $1 
                WHERE id = $2
                "#,
                serde_json::to_value(preview).ok() as _,
                link_id
            )
            .execute(&pool_clone)
            .await;
        }
    });

    // Return the created link immediately
    let response = ApiResponse::success_with_message(link, "Link created successfully");
    (StatusCode::CREATED, Json(response)).into_response()
}

/// Track a link click
///
/// Increments the click count for a link
pub async fn track_click(
    State(pool): State<PgPool>,
    Path(link_id): Path<Uuid>,
) -> impl IntoResponse {
    match increment_click_count(&pool, link_id).await {
        Ok(_) => {
            let response = ApiResponse::success(());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse::new(format!("Failed to track click: {e}"))
                .with_code("CLICK_TRACK_ERROR");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Delete a link
///
/// Delete a link by its ID. This operation requires authentication and can only be performed by the link's owner.
///
/// # OpenAPI Specification
/// ```yaml
/// /api/links/{id}:
///   delete:
///     summary: Delete a link
///     description: Delete a link by its ID. Only the owner of the link can delete it.
///     tags:
///       - links
///     security:
///       - bearerAuth: []
///     parameters:
///       - name: id
///         in: path
///         required: true
///         description: Numeric ID of the link to delete
///         schema:
///           type: integer
///           format: int32
///     responses:
///       200:
///         description: Link successfully deleted
///         content:
///           application/json:
///             schema:
///               type: object
///               properties:
///                 success:
///                   type: boolean
///                   example: true
///                 message:
///                   type: string
///                   example: Link deleted successfully
///       401:
///         description: Unauthorized - Missing or invalid JWT token
///         content:
///           application/json:
///             schema:
///               type: object
///               properties:
///                 error:
///                   type: string
///                   example: Missing or invalid authorization header
///                 code:
///                   type: string
///                   example: UNAUTHORIZED
///       403:
///         description: Forbidden - User doesn't own the link
///         content:
///           application/json:
///             schema:
///               type: object
///               properties:
///                 error:
///                   type: string
///                   example: You don't have permission to delete this link
///                 code:
///                   type: string
///                   example: FORBIDDEN
///       404:
///         description: Link not found
///         content:
///           application/json:
///             schema:
///               type: object
///               properties:
///                 error:
///                   type: string
///                   example: Link not found
///                 code:
///                   type: string
///                   example: NOT_FOUND
///       500:
///         description: Internal server error
///         content:
///           application/json:
///             schema:
///               type: object
///               properties:
///                 error:
///                   type: string
///                   example: Failed to delete link
///                 code:
///                   type: string
///                   example: LINK_DELETE_ERROR
/// ```
pub async fn delete_link(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(link_id): Path<Uuid>,
) -> impl IntoResponse {
    // First check if the link exists and belongs to the user
    match database::queries::get_link_by_id(&pool, link_id).await {
        Ok(Some(link)) => {
            if link.user_id != user.id {
                let error = ErrorResponse::new("You don't have permission to delete this link")
                    .with_code("FORBIDDEN");
                return (StatusCode::FORBIDDEN, Json(error)).into_response();
            }

            // If the user owns the link, proceed with deletion
            match database::queries::delete_link(&pool, link_id).await {
                Ok(_) => {
                    let response =
                        ApiResponse::success_with_message((), "Link deleted successfully");
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(e) => {
                    let error = ErrorResponse::new(format!("Failed to delete link: {e}"))
                        .with_code("LINK_DELETE_ERROR");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                }
            }
        }
        Ok(None) => {
            let error = ErrorResponse::new("Link not found").with_code("NOT_FOUND");
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse::new(format!("Failed to fetch link: {e}"))
                .with_code("LINK_FETCH_ERROR");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Update a link by ID
///
/// Only the owner can update. Returns the updated link or error.
#[utoipa::path(
    put,
    path = "/api/links/{id}",
    request_body = CreateLinkRequest,
    responses(
        (status = 200, description = "Link updated successfully", body = LinkResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Link not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    ),
    tag = "links"
)]
pub async fn update_link_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthUser>,
    Path(link_id): Path<Uuid>,
    Json(payload): Json<CreateLinkRequest>,
) -> impl IntoResponse {
    // Validate the request payload
    if let Err(validation_errors) = payload.validate() {
        let error = ErrorResponse::new(format!("Validation error: {validation_errors}")).with_code("VALIDATION_ERROR");
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(error)).into_response();
    }
    // Check ownership
    match get_link_by_id(&pool, link_id).await {
        Ok(Some(link)) => {
            if link.user_id != user.id {
                let error = ErrorResponse::new("You don't have permission to update this link").with_code("FORBIDDEN");
                return (StatusCode::FORBIDDEN, Json(error)).into_response();
            }
            // Update the link
            match update_link(&pool, link_id, payload.url, payload.title, payload.description).await {
                Ok(Some(updated_link)) => {
                    let response = ApiResponse::success_with_message(updated_link, "Link updated successfully");
                    (StatusCode::OK, Json(response)).into_response()
                }
                Ok(None) => {
                    let error = ErrorResponse::new("Link not found").with_code("NOT_FOUND");
                    (StatusCode::NOT_FOUND, Json(error)).into_response()
                }
                Err(e) => {
                    let error = ErrorResponse::new(format!("Failed to update link: {e}")).with_code("LINK_UPDATE_ERROR");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                }
            }
        }
        Ok(None) => {
            let error = ErrorResponse::new("Link not found").with_code("NOT_FOUND");
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse::new(format!("Failed to fetch link: {e}")).with_code("LINK_FETCH_ERROR");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Get a single link by ID
///
/// Returns the link with the given ID, or 404 if not found
#[utoipa::path(
    get,
    path = "/api/links/{id}",
    responses(
        (status = 200, description = "Link retrieved successfully", body = LinkResponse),
        (status = 404, description = "Link not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    ),
    tag = "links"
)]
pub async fn get_link_by_id_handler(
    State(pool): State<PgPool>,
    Path(link_id): Path<Uuid>,
) -> impl IntoResponse {
    match get_link_by_id(&pool, link_id).await {
        Ok(Some(link)) => {
            let response = ApiResponse::success(link);
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let error = ErrorResponse::new("Link not found").with_code("NOT_FOUND");
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse::new(format!("Failed to fetch link: {e}")).with_code("LINK_FETCH_ERROR");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

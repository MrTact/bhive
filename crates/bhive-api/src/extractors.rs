//! Custom extractors for API handlers

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

/// Extracts the project ID from the X-Project-ID header
pub struct ProjectId(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for ProjectId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let project_id = parts
            .headers
            .get("X-Project-ID")
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Missing X-Project-ID header.\n\n\
                     This endpoint requires a project ID. Make sure you've run 'bhive init' \
                     in your project directory.\n\n\
                     The CLI should automatically include this header. If you're calling the API \
                     directly, add the header:\n  \
                     X-Project-ID: your_project_id"
                        .to_string(),
                )
            })?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid X-Project-ID header value".to_string(),
                )
            })?
            .to_string();

        Ok(ProjectId(project_id))
    }
}

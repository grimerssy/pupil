use crate::http::error::HttpError;
use axum::http::StatusCode;

use super::view::View;

#[derive(Debug, thiserror::Error)]
#[error("Requested resourse was not found")]
pub struct RouteNotFound;

impl HttpError for RouteNotFound {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

#[tracing::instrument]
pub async fn not_found_view() -> View<RouteNotFound> {
    let error = RouteNotFound;
    tracing::info!(?error);
    View::error(error)
}

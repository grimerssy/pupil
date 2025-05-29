use crate::{
    app::localization::LocalizedError,
    http::{error::HttpError, middleware::template::TemplateName},
};
use axum::http::StatusCode;

use super::view::View;

#[derive(Debug)]
pub struct RouteNotFound;

impl HttpError for RouteNotFound {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

impl From<RouteNotFound> for LocalizedError {
    fn from(_: RouteNotFound) -> Self {
        Self::new("NOT_FOUND")
    }
}

#[tracing::instrument]
pub async fn not_found_view() -> View<crate::Error<RouteNotFound>> {
    let error = crate::Error::expected(RouteNotFound);
    tracing::info!(?error);
    View::new(TemplateName::error(), error)
}

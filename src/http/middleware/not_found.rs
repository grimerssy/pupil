use crate::{
    app::error::{ContextualError, ErrorContext},
    http::{error::HttpError, middleware::template::TemplateName, response::Rejection},
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

impl ContextualError for RouteNotFound {
    fn error_context(self) -> ErrorContext {
        ErrorContext::new("NOT_FOUND")
    }
}

#[tracing::instrument]
pub async fn not_found_view() -> View<Rejection<RouteNotFound>> {
    let error = RouteNotFound;
    tracing::info!(?error);
    View::new(TemplateName::error(), Rejection(error))
}

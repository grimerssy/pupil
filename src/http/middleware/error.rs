use std::any::Any;

use anyhow::anyhow;
use axum::{
    body::to_bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::Error;

use super::view::{ErrorView, View};

#[derive(Clone, Debug, thiserror::Error)]
#[error("{body}")]
pub struct ExtractionError {
    status: StatusCode,
    body: String,
}

impl ExtractionError {
    /// # Panics
    ///
    /// Panics if IntoResponse doesn't uphold the same invariants as axum rejection types:
    /// - reading the body is infallible
    /// - body is a valid UTF-8 sequence
    pub(super) async fn from_rejection(rejection: impl IntoResponse) -> Self {
        Self::from_response(rejection.into_response())
            .await
            .expect("receive axum rejection")
    }

    /// # Errors
    ///
    /// Returns error if:
    /// - reading the body fails
    /// - reference to the body isn't exclusive
    /// - body isn't a valid UTF-8
    async fn from_response(response: Response) -> anyhow::Result<Self> {
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await?;
        let unique_bytes = bytes
            .try_into_mut()
            .map(Vec::from) // exclusive reference ensures zero copy conversion
            .map_err(|_| anyhow!("non-exclusive reference to the body"))?;
        let body_text = String::from_utf8(unique_bytes)?;
        Ok(Self {
            status: parts.status,
            body: body_text,
        })
    }
}

pub(super) async fn handle_not_found() -> ErrorView {
    #[tracing::instrument(ret(level = "warn"))]
    fn not_found() -> Error {
        Error::NotFound
    }
    View::error(not_found())
}

pub(super) fn handle_panic(panic_message: Box<dyn Any + Send + 'static>) -> Response {
    #[tracing::instrument(skip_all, ret(level = "error"))]
    fn catch_panic(panic_message: Box<dyn Any + Send + 'static>) -> Error {
        let error = if let Some(msg) = panic_message.downcast_ref::<String>() {
            anyhow!("{msg}")
        } else if let Some(msg) = panic_message.downcast_ref::<&str>() {
            anyhow!("{msg}")
        } else {
            anyhow!("unknown panic message")
        }
        .context("catch panic");
        Error::from(error)
    }
    View::error(catch_panic(panic_message)).into_response()
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ErrorMessage {
    errors: Vec<String>,
}

pub(super) fn error_response<T>(
    error: &Error,
    into_body: impl FnOnce(ErrorMessage) -> T,
) -> Response
where
    T: IntoResponse,
{
    let msg = create_message(error);
    let body = into_body(msg).into_response();
    create_response(error, body)
}

fn create_message(error: &Error) -> ErrorMessage {
    let errors = vec![error.to_string()];
    ErrorMessage { errors }
}

fn create_response(error: &Error, body: Response) -> Response {
    (status_code(error), body).into_response()
}

fn status_code(error: &Error) -> StatusCode {
    match error {
        Error::NotFound => StatusCode::NOT_FOUND,
        Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Error::HttpExtraction(e) => e.status,
    }
}

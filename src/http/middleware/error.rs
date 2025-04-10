use std::any::Any;

use anyhow::anyhow;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::Error;

use super::view::{ErrorView, View};

pub async fn handle_not_found() -> ErrorView {
    #[tracing::instrument(level = "trace", ret(level = "warn"))]
    fn not_found() -> Error {
        Error::NotFound
    }
    View::error(not_found())
}

pub fn handle_panic(panic_message: Box<dyn Any + Send + 'static>) -> Response {
    #[tracing::instrument(level = "trace", skip_all, ret(level = "error"))]
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
    error: String,
}

pub(super) fn error_response<T>(
    error: &Error,
    into_body: impl FnOnce(ErrorMessage) -> T,
) -> Response
where
    T: IntoResponse,
{
    let msg = ErrorMessage {
        error: error.to_string(),
    };
    let body = into_body(msg).into_response();
    create_response(error, body)
}

fn create_response(error: &Error, body: Response) -> Response {
    (status_code(error), body).into_response()
}

fn status_code(error: &Error) -> StatusCode {
    match error {
        Error::NotFound => StatusCode::NOT_FOUND,
        Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

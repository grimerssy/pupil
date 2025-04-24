use std::{any::Any, convert::Infallible};

use anyhow::anyhow;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::error::{Error, InternalError};

use super::view::{ErrorView, View};

pub trait HttpError: std::error::Error {
    fn status_code(&self) -> StatusCode;
}

#[derive(Debug, thiserror::Error)]
#[error("Requested resourse was not found")]
pub(super) struct RouteNotFound;

impl HttpError for RouteNotFound {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

pub(super) async fn handle_not_found() -> ErrorView<RouteNotFound> {
    #[tracing::instrument(ret(level = "warn"))]
    fn not_found() -> RouteNotFound {
        RouteNotFound
    }
    let error = Error::Domain(not_found());
    View::error(error)
}

pub(super) fn handle_panic(panic_message: Box<dyn Any + Send + 'static>) -> Response {
    #[tracing::instrument(skip_all, ret(level = "error"))]
    fn catch_panic(panic_message: Box<dyn Any + Send + 'static>) -> InternalError {
        if let Some(msg) = panic_message.downcast_ref::<String>() {
            anyhow!("{msg}")
        } else if let Some(msg) = panic_message.downcast_ref::<&str>() {
            anyhow!("{msg}")
        } else {
            anyhow!("unknown panic message")
        }
        .context("catch panic")
        .into()
    }
    let error = catch_panic(panic_message);
    let error = Error::<Infallible>::Unexpected(error);
    View::error(error).into_response()
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ErrorMessage {
    error: String,
}

pub(super) fn error_response<R>(
    error: &impl HttpError,
    into_body: impl FnOnce(ErrorMessage) -> R,
) -> Response
where
    R: IntoResponse,
{
    let msg = create_message(error);
    let body = into_body(msg).into_response();
    create_response(error, body)
}

fn create_message(error: &impl std::error::Error) -> ErrorMessage {
    let error = error.to_string();
    ErrorMessage { error }
}

fn create_response(error: &impl HttpError, body: Response) -> Response {
    (error.status_code(), body).into_response()
}

impl HttpError for Infallible {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl<E> HttpError for Error<E>
where
    E: HttpError,
{
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Domain(e) => e.status_code(),
        }
    }
}

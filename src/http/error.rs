use axum::http::StatusCode;
use serde::Serialize;

use crate::{
    app::error::{AppError, AppErrorKind},
    domain::error::{DomainError, InternalError},
};

pub trait HttpError {
    fn status_code(&self) -> StatusCode;
}

impl HttpError for InternalError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<I, E> HttpError for AppError<I, E>
where
    I: Serialize + Send + Sync + 'static,
    E: HttpError,
{
    fn status_code(&self) -> StatusCode {
        match &self.kind {
            AppErrorKind::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppErrorKind::Logical(DomainError::Internal(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorKind::Logical(DomainError::Expected(e)) => e.status_code(),
        }
    }
}

use std::convert::Infallible;

use axum::http::StatusCode;

use crate::{
    app::error::{AppError, AppErrorKind},
    domain::error::DomainError,
};

pub trait HttpError {
    fn status_code(&self) -> StatusCode;
}

impl<I, E> HttpError for AppError<I, E>
where
    E: HttpError,
{
    fn status_code(&self) -> StatusCode {
        match &self.kind {
            AppErrorKind::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppErrorKind::Logical(error) => error.status_code(),
        }
    }
}

impl<E> HttpError for DomainError<E>
where
    E: HttpError,
{
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Expected(error) => error.status_code(),
        }
    }
}

impl HttpError for Infallible {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

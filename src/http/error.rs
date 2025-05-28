use std::convert::Infallible;

use axum::http::StatusCode;

use crate::{app::error::AppError, error::ErrorKind};

pub trait HttpError {
    fn status_code(&self) -> StatusCode;
}

impl HttpError for Infallible {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl<E> HttpError for crate::Error<E>
where
    E: HttpError,
{
    fn status_code(&self) -> StatusCode {
        match &self.kind {
            ErrorKind::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::Expected(error) => error.status_code(),
        }
    }
}

impl<E> HttpError for AppError<E>
where
    E: HttpError,
{
    fn status_code(&self) -> StatusCode {
        match &self {
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Logical(error) => error.status_code(),
        }
    }
}

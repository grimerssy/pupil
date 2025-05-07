use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{
    app::error::{AppError, AppErrorKind},
    domain::error::{DomainError, InternalError},
    http::response::{ErrorHttpResponse, HttpResponseExtension},
};

pub trait HttpError: std::error::Error {
    fn status_code(&self) -> StatusCode;

    fn into_http_response(self) -> HttpResponseExtension
    where
        Self: Sized,
    {
        ErrorHttpResponse::Error {
            input: (),
            message: self.to_string(),
        }
        .erase_types()
    }

    fn with_body<R>(self, into_body: impl FnOnce(HttpResponseExtension) -> R) -> Response
    where
        Self: Sized,
        R: IntoResponse,
    {
        let status_code = self.status_code();
        let response = self.into_http_response();
        let body = into_body(response);
        (status_code, body).into_response()
    }
}

impl HttpError for InternalError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<I, E> HttpError for AppError<I, E>
where
    I: Serialize + Send + Sync + 'static,
    E: HttpError + std::error::Error,
{
    fn status_code(&self) -> StatusCode {
        match &self.kind {
            AppErrorKind::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppErrorKind::Logical(DomainError::Internal(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorKind::Logical(DomainError::Expected(e)) => e.status_code(),
        }
    }

    fn into_http_response(self) -> HttpResponseExtension
    where
        Self: Sized,
    {
        let AppError { input, kind } = self;
        match kind {
            AppErrorKind::Validation(errors) => ErrorHttpResponse::Fail {
                input,
                data: errors,
            },
            AppErrorKind::Logical(error) => ErrorHttpResponse::Error {
                input,
                message: error.to_string(),
            },
        }
        .erase_types()
    }
}

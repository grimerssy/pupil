use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::app::{
    error::{AppError, ContextualError, ErrorContext},
    validation::ValidationErrors,
};

use super::error::HttpError;

pub trait ResponseContext {
    fn with_body<F, R>(self, to_body: F) -> Response
    where
        F: FnOnce(HttpResponse) -> R,
        R: IntoResponse;
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum HttpResponse {
    Success {
        data: OpaqueData,
    },
    Fail {
        input: OpaqueData,
        data: ValidationErrors,
    },
    Error {
        input: OpaqueData,
        data: ErrorContext,
    },
}

pub struct Success<T>(pub T);

pub struct Rejection<E>(pub E);

#[derive(Serialize)]
pub struct OpaqueData(Box<dyn erased_serde::Serialize + Send + Sync>);

impl<T> ResponseContext for Success<T>
where
    T: Serialize + Send + Sync + 'static,
{
    fn with_body<F, R>(self, to_body: F) -> Response
    where
        F: FnOnce(HttpResponse) -> R,
        R: IntoResponse,
    {
        let response = HttpResponse::Success {
            data: OpaqueData(Box::new(self.0)),
        };
        to_body(response).into_response()
    }
}

impl<E> ResponseContext for Rejection<E>
where
    E: HttpError + ContextualError,
{
    fn with_body<F, R>(self, to_body: F) -> Response
    where
        F: FnOnce(HttpResponse) -> R,
        R: IntoResponse,
    {
        let error = self.0;
        let status_code = error.status_code();
        let response = HttpResponse::Error {
            input: OpaqueData(Box::new(())),
            data: error.error_context(),
        };
        let body = to_body(response).into_response();
        (status_code, body).into_response()
    }
}

impl<I, E> ResponseContext for crate::error::Rejection<I, AppError<E>>
where
    Self: HttpError,
    I: Serialize + Send + Sync + 'static,
    E: ContextualError,
{
    fn with_body<F, R>(self, to_body: F) -> Response
    where
        F: FnOnce(HttpResponse) -> R,
        R: IntoResponse,
    {
        let status_code = self.status_code();
        let input = OpaqueData(Box::new(self.input));
        let response = match self.error {
            AppError::Validation(errors) => HttpResponse::Fail {
                input,
                data: errors,
            },
            AppError::Logical(error) => HttpResponse::Error {
                input,
                data: error.error_context(),
            },
        };
        let body = to_body(response).into_response();
        (status_code, body).into_response()
    }
}

impl Clone for HttpResponse {
    fn clone(&self) -> Self {
        unreachable!("HTTP response may not be cloned")
    }
}

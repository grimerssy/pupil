use serde::Serialize;

use crate::{
    app::{localization::LocalizedError, validation::ValidationErrors, AppError},
    error::ErrorKind,
};

pub struct HttpResponse {
    pub message: Box<dyn erased_serde::Serialize + Send + Sync>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum HttpMessage<T> {
    Success {
        data: T,
    },
    Fail {
        input: T,
        data: ValidationErrors,
    },
    Error {
        input: T,
        data: LocalizedError,
    },
}

pub enum ErrorType {
    Fail(ValidationErrors),
    Error(LocalizedError),
}

impl HttpResponse {
    pub fn success<T>(data: T) -> Self
    where
        T: Serialize + Send + Sync + 'static,
    {
        let message = Box::new(HttpMessage::Success { data });
        Self { message }
    }

    pub fn error<E, I>(error: crate::Error<E, I>) -> Self
    where
        E: Into<ErrorType>,
        I: Serialize + Send + Sync + 'static,
    {
        let input = error.input;
        let error_type = ErrorType::new(error.kind);
        let message = Box::new(match error_type {
            ErrorType::Fail(data) => HttpMessage::Fail { input, data },
            ErrorType::Error(data) => HttpMessage::Error { input, data },
        });
        Self { message }
    }
}

impl ErrorType {
    fn new<E>(error: ErrorKind<E>) -> Self
    where
        E: Into<Self>,
    {
        match error {
            ErrorKind::Expected(error) => error.into(),
            ErrorKind::Internal(error) => <ErrorKind>::Internal(error).into(),
        }
    }
}

impl<E> From<E> for ErrorType
where
    E: Into<LocalizedError>,
{
    fn from(value: E) -> Self {
        Self::Error(value.into())
    }
}

impl<E> From<AppError<E>> for ErrorType
where
    E: Into<LocalizedError>,
{
    fn from(value: AppError<E>) -> Self {
        match value {
            AppError::Validation(errors) => Self::Fail(errors),
            AppError::Logical(error) => Self::from(error),
        }
    }
}

impl Clone for HttpResponse {
    fn clone(&self) -> Self {
        unreachable!("HTTP response may not be cloned")
    }
}

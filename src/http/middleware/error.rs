use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::Error;

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
    error: String,
}

pub fn error_response<T>(error: &Error, into_body: impl FnOnce(ErrorMessage) -> T) -> Response
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

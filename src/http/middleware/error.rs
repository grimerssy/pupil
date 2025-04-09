use axum::http::StatusCode;

use crate::Error;

#[derive(Clone, Debug)]
pub struct ErrorResponse {
    pub status_code: StatusCode,
    pub message: ErrorMessage,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
    error: String,
}

impl ErrorResponse {
    pub fn new(error: &Error) -> Self {
        Self {
            status_code: status_code(error),
            message: ErrorMessage::new(error),
        }
    }
}

fn status_code(error: &Error) -> StatusCode {
    match error {
        Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl ErrorMessage {
    fn new(error: &Error) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

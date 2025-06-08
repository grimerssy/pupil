use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::http::error::HttpError;

use super::response::{ErrorType, HttpResponse};

pub struct Json<T>(pub T);

impl IntoResponse for Json<HttpResponse> {
    fn into_response(self) -> Response {
        axum::Json(self.0.message).into_response()
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let response = HttpResponse::success(self.0);
        Json(response).into_response()
    }
}

impl<E, I> IntoResponse for Json<crate::Error<E, I>>
where
    E: HttpError + Into<ErrorType>,
    I: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let status = self.0.kind.status_code();
        let response = HttpResponse::error(self.0);
        (status, Json(response)).into_response()
    }
}

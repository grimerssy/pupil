use std::borrow::Cow;

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Serialize;

use super::{
    error::ErrorResponse,
    response_type::{LazyResponseType, ResponseType},
    template::{Template, ERROR_TEMPLATE},
};

#[derive(Clone, Debug)]
pub struct View<T> {
    template_name: Cow<'static, str>,
    data: T,
}

#[derive(Debug)]
pub struct ErrorView(pub crate::Error);

pub type ResultView<T> = Result<View<T>, ErrorView>;

struct SealedData(Box<dyn erased_serde::Serialize + Send + Sync>);

impl<T> View<T> {
    pub fn new<N>(template_name: N, data: T) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        Self {
            template_name: template_name.into(),
            data,
        }
    }
}

pub async fn render_view(response_type: LazyResponseType, req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let Some(view) = response.extensions_mut().remove::<View<SealedData>>() else {
        return response;
    };
    // parts can't contain content-type so no need to sanitize
    // - only one IntoResponse can be contained in Response
    // - SealedData is a private type
    let (parts, _) = response.into_parts();
    let response = match response_type.parse() {
        ResponseType::Json => Json(view.data.0).into_response(),
        ResponseType::Html => Template::new(view.template_name, view.data.0).into_response(),
    };
    (parts, response).into_response()
}

impl Clone for View<SealedData> {
    fn clone(&self) -> Self {
        unreachable!("a view body may not be cloned")
    }
}

impl<T> IntoResponse for View<T>
where
    T: Serialize + Clone + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let view = View {
            template_name: self.template_name,
            data: SealedData(Box::new(self.data)),
        };
        Extension(view).into_response()
    }
}

impl IntoResponse for ErrorView {
    fn into_response(self) -> Response {
        let error = ErrorResponse::new(&self.0);
        let view = View::new(ERROR_TEMPLATE, error.message);
        (error.status_code, view).into_response()
    }
}

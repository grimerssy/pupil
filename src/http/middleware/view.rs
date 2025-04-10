use std::borrow::Cow;

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Serialize;

use super::{
    error::error_response,
    response_type::{LazyResponseType, ResponseType},
    template::{Template, TemplateMeta},
};

#[derive(Clone, Debug)]
pub struct View<T> {
    template_meta: TemplateMeta,
    data: T,
}

pub type ResultView<T> = Result<View<T>, ErrorView>;

pub type ErrorView = View<crate::Error>;

struct SealedData(Box<dyn erased_serde::Serialize + Send + Sync>);

impl<T> View<T> {
    pub fn new<N>(template_name: N, data: T) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        let template_meta = TemplateMeta::new(template_name);
        Self {
            template_meta,
            data,
        }
    }
}

impl View<crate::Error> {
    pub fn error(error: crate::Error) -> Self {
        Self {
            template_meta: TemplateMeta::error(),
            data: error,
        }
    }
}

pub async fn render_view(response_type: LazyResponseType, req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let Some(view) = response.extensions_mut().remove::<View<SealedData>>() else {
        return response;
    };
    let body = match response_type.parse() {
        ResponseType::Json => Json(view.data.0).into_response(),
        ResponseType::Html => Template::with_meta(view.template_meta, view.data.0).into_response(),
    };
    if !body.status().is_success() {
        return body;
    }
    // parts can't contain *implicitly set* content-type so no need to sanitize
    // - only one IntoResponse can be contained in Response and that's View
    // - SealedData is a private type, hence can't be injected via Extension from outside
    let (parts, _) = response.into_parts();
    (parts, body).into_response()
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
            template_meta: self.template_meta,
            data: SealedData(Box::new(self.data)),
        };
        Extension(view).into_response()
    }
}

impl IntoResponse for View<crate::Error> {
    fn into_response(self) -> Response {
        let into_view = |msg| View {
            template_meta: self.template_meta,
            data: msg,
        };
        error_response(&self.data, into_view)
    }
}

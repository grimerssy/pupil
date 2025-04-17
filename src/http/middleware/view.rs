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

impl<T> View<T> {
    pub fn new(template_name: impl Into<Cow<'static, str>>, data: T) -> Self {
        let template_meta = TemplateMeta::new(template_name);
        Self {
            template_meta,
            data,
        }
    }
}

impl ErrorView {
    pub fn error(error: crate::Error) -> Self {
        Self {
            template_meta: TemplateMeta::error(),
            data: error,
        }
    }
}

struct Private<T>(T);

type OpaqueData = Box<dyn erased_serde::Serialize + Send + Sync>;

pub(super) async fn render_view(response_type: LazyResponseType, req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let Some(view) = response
        .extensions_mut()
        .remove::<View<Private<OpaqueData>>>()
    else {
        return response;
    };
    let (parts, _) = response.into_parts();
    match response_type.parse() {
        ResponseType::Json => (parts, Json(view.data.0)).into_response(),
        ResponseType::Html => {
            (parts, Template::with_meta(view.template_meta, view.data.0)).into_response()
        }
    }
}

impl Clone for View<Private<OpaqueData>> {
    fn clone(&self) -> Self {
        unreachable!("a view body may not be cloned")
    }
}

impl<T> IntoResponse for View<T>
where
    T: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let opaque_data: OpaqueData = Box::new(self.data);
        let view = View {
            template_meta: self.template_meta,
            data: Private(opaque_data),
        };
        Extension(view).into_response()
    }
}

impl IntoResponse for ErrorView {
    fn into_response(self) -> Response {
        let into_view = |msg| View {
            template_meta: self.template_meta,
            data: msg,
        };
        error_response(&self.data, into_view)
    }
}

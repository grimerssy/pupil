use std::borrow::Cow;

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Serialize;

use crate::error::Error;

use super::{
    error::{error_response, HttpError},
    response_type::{LazyResponseType, ResponseType},
    template::{Template, TemplateMeta},
};

#[derive(Clone, Debug)]
pub struct View<T> {
    template_meta: TemplateMeta,
    data: T,
}

pub type ResultView<T, E> = Result<View<T>, ErrorView<E>>;

pub type ErrorView<E> = View<Error<E>>;

impl<T> View<T> {
    pub fn new(template_name: impl Into<Cow<'static, str>>, data: T) -> Self {
        let template_meta = TemplateMeta::new(template_name);
        Self::with_meta(template_meta, data)
    }

    pub fn error(error: T) -> Self {
        let template_meta = TemplateMeta::error();
        Self::with_meta(template_meta, error)
    }

    fn with_meta(template_meta: TemplateMeta, data: T) -> Self {
        Self {
            template_meta,
            data,
        }
    }
}

struct Private<T>(T);

type OpaqueData = Box<dyn erased_serde::Serialize + Send + Sync>;

pub(super) async fn handle_render_view(
    response_type: LazyResponseType,
    req: Request,
    next: Next,
) -> Response {
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
        let view = View::with_meta(self.template_meta, Private(opaque_data));
        Extension(view).into_response()
    }
}

impl<E> IntoResponse for ErrorView<E>
where
    E: HttpError,
{
    fn into_response(self) -> Response {
        let into_view = |msg| View::with_meta(self.template_meta, msg);
        error_response(&self.data, into_view)
    }
}

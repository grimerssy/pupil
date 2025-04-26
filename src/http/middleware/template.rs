use std::{borrow::Cow, convert::Infallible};

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde::Serialize;

use crate::{context::AppContext, error::Error, template::render_template};

use super::error::{error_response, HttpError};

#[derive(Clone, Debug)]
pub struct Template<T> {
    meta: TemplateMeta,
    data: T,
}

pub type ErrorTemplate<E> = Template<Error<E>>;

#[derive(Clone, Debug)]
pub struct TemplateMeta {
    name: Cow<'static, str>,
}

impl<T> Template<T> {
    pub fn error(error: T) -> Self {
        let meta = TemplateMeta::error();
        Self::with_meta(meta, error)
    }

    pub fn with_meta(meta: TemplateMeta, data: T) -> Self {
        Self { meta, data }
    }
}

impl TemplateMeta {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        Self { name }
    }

    pub fn error() -> Self {
        Self::new("error.html")
    }
}

struct Private<T>(T);

type OpaqueData = Box<dyn erased_serde::Serialize + Send + Sync>;

fn seal_data<T>(data: T) -> Private<OpaqueData>
where
    T: Serialize + Send + Sync + 'static,
{
    Private(Box::new(data))
}

pub(super) async fn handle_render_template(
    State(ctx): State<AppContext>,
    req: Request,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;
    let Some(template) = response
        .extensions_mut()
        .remove::<Template<Private<OpaqueData>>>()
    else {
        return response;
    };
    let html = match render_template(&ctx, &template.meta.name, template.data.0) {
        Ok(html) => html,
        Err(error) => {
            let error = Error::<Infallible>::Unexpected(error);
            response = Template::error(error).into_response();
            let template = response
                .extensions_mut()
                .remove::<Template<Private<OpaqueData>>>()
                .unwrap();
            render_template(&ctx, &template.meta.name, template.data.0)
                .expect("render error template")
        }
    };
    let (parts, _) = response.into_parts();
    (parts, Html(html)).into_response()
}

impl Clone for Template<Private<OpaqueData>> {
    fn clone(&self) -> Self {
        unreachable!("a template body may not be cloned")
    }
}

impl<T> IntoResponse for Template<T>
where
    T: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let template = Template::with_meta(self.meta, seal_data(self.data));
        Extension(template).into_response()
    }
}

impl<E> IntoResponse for ErrorTemplate<E>
where
    E: HttpError,
{
    fn into_response(self) -> Response {
        let into_template = |msg| Template::with_meta(self.meta, msg);
        error_response(&self.data, into_template)
    }
}

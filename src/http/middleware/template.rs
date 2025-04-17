use std::borrow::Cow;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde::Serialize;

use crate::context::AppContext;

use super::error::error_response;

#[derive(Clone, Debug)]
pub struct Template<T> {
    meta: TemplateMeta,
    data: T,
}

pub type ErrorTemplate = Template<crate::Error>;

#[derive(Clone, Debug)]
pub struct TemplateMeta {
    name: Cow<'static, str>,
}

impl<T> Template<T> {
    pub fn with_meta(meta: TemplateMeta, data: T) -> Self {
        Self { meta, data }
    }
}

impl ErrorTemplate {
    pub fn error(error: crate::Error) -> Self {
        let meta = TemplateMeta::error();
        Self::with_meta(meta, error)
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

pub(super) async fn render_template(
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
    let html = match ctx.render_template(&template.meta.name, template.data.0) {
        Ok(html) => html,
        Err(error) => {
            response = Template::error(error).into_response();
            let template = response
                .extensions_mut()
                .remove::<Template<Private<OpaqueData>>>()
                .unwrap();
            ctx.render_template(&template.meta.name, template.data.0)
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

impl IntoResponse for ErrorTemplate {
    fn into_response(self) -> Response {
        let into_template = |msg| Template::with_meta(self.meta, msg);
        error_response(&self.data, into_template)
    }
}

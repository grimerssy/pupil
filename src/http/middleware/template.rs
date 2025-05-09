use std::borrow::Cow;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde::Serialize;

use crate::{
    app::error::AppError,
    context::AppContext,
    domain::error::InternalError,
    http::{
        error::HttpError,
        response::{HttpResponse, HttpResponseExtension, SuccessHttpResponse},
    },
};

pub trait RenderTemplate {
    fn render_template<T>(&self, template_name: &str, data: T) -> Result<String, InternalError>
    where
        T: Serialize;
}

#[derive(Clone, Debug)]
pub struct Template<T> {
    meta: TemplateMeta,
    data: T,
}

pub type SuccessTemplate<T> = Template<SuccessHttpResponse<T>>;

pub type ErrorTemplate<I, E> = Template<AppError<I, E>>;

pub(super) type TemplateName = Cow<'static, str>;

#[derive(Clone, Debug)]
pub struct TemplateMeta {
    name: TemplateName,
}

impl<T> Template<T> {
    pub fn new(template_name: impl Into<TemplateName>, data: T) -> Self {
        let template_meta = TemplateMeta::new(template_name);
        Self::with_meta(template_meta, data)
    }

    pub fn error(error: T) -> Self {
        let meta = TemplateMeta::error();
        Self::with_meta(meta, error)
    }

    pub fn with_meta(meta: TemplateMeta, data: T) -> Self {
        Self { meta, data }
    }
}

impl TemplateMeta {
    pub fn new(name: impl Into<TemplateName>) -> Self {
        let name = name.into();
        Self { name }
    }

    pub fn error() -> Self {
        Self::new("error.html")
    }
}

pub(super) async fn handle_render_template(
    State(ctx): State<AppContext>,
    req: Request,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;
    let Some(template) = response
        .extensions_mut()
        .remove::<Template<HttpResponseExtension>>()
    else {
        return response;
    };
    let html = match ctx.render_template(&template.meta.name, template.data) {
        Ok(html) => html,
        Err(error) => {
            response = Template::error(error).into_response();
            let template = response
                .extensions_mut()
                .remove::<Template<HttpResponseExtension>>()
                .unwrap();
            ctx.render_template(&template.meta.name, template.data)
                .expect("render error template")
        }
    };
    let (parts, _) = response.into_parts();
    (parts, Html(html)).into_response()
}

impl<I, O, V> IntoResponse for Template<HttpResponse<I, O, V>>
where
    I: Serialize + Send + Sync + 'static,
    O: Serialize + Send + Sync + 'static,
    V: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let view = Template::with_meta(self.meta, self.data.erase_types());
        Extension(view).into_response()
    }
}

impl<E> IntoResponse for Template<E>
where
    E: HttpError,
{
    fn into_response(self) -> Response {
        self.data
            .with_body(|response| Template::with_meta(self.meta, response))
    }
}

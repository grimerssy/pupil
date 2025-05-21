use std::borrow::Cow;

use axum::{
    extract::{Request, State},
    http::header::ACCEPT_LANGUAGE,
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde::Serialize;
use unic_langid::LanguageIdentifier;

use crate::{
    app::AppContext,
    domain::error::InternalError,
    http::response::{HttpResponse, Rejection, ResponseContext},
};

pub trait TemplateRenderer {
    fn render_template<T>(
        &self,
        template_name: &str,
        data: T,
        lang: &LanguageIdentifier,
    ) -> Result<String, InternalError>
    where
        T: Serialize;
}

pub trait LanguageNegotiator {
    fn negotiate_language(&self, sorted_preferences: Vec<LanguageIdentifier>)
        -> LanguageIdentifier;
}

#[derive(Clone, Debug)]
pub struct Template<T> {
    meta: TemplateMeta,
    data: T,
}

pub(super) type TemplateName = Cow<'static, str>;

#[derive(Clone, Debug)]
pub struct TemplateMeta {
    name: TemplateName,
}

impl<T> Template<T> {
    pub fn new(template_name: impl Into<TemplateName>, data: T) -> Self {
        let meta = TemplateMeta::new(template_name);
        Self::with_meta(meta, data)
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

pub(super) async fn render_template(
    State(ctx): State<AppContext>,
    req: Request,
    next: Next,
) -> Response {
    let accept_language_header = req.headers().get(ACCEPT_LANGUAGE).cloned();
    let mut response = next.run(req).await;
    let Some(template) = response.extensions_mut().remove::<Template<HttpResponse>>() else {
        return response;
    };
    let renderer = ctx.templating_engine;
    let language_preferences = accept_language_header
        .and_then(|header| header.to_str().ok().map(accept_language::parse))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|lang| lang.parse::<LanguageIdentifier>().ok())
        .collect::<Vec<_>>();
    let lang = ctx.localizer.negotiate_language(language_preferences);
    let html = match renderer.render_template(&template.meta.name, template.data, &lang) {
        Ok(html) => html,
        Err(error) => {
            response = Template::error(Rejection(error)).into_response();
            let template = response
                .extensions_mut()
                .remove::<Template<HttpResponse>>()
                .unwrap();
            renderer
                .render_template(&template.meta.name, template.data, &lang)
                .expect("render error template")
        }
    };
    let (parts, _) = response.into_parts();
    (parts, Html(html)).into_response()
}

impl IntoResponse for Template<HttpResponse> {
    fn into_response(self) -> Response {
        Extension(self).into_response()
    }
}

impl<T> IntoResponse for Template<T>
where
    T: ResponseContext,
{
    fn into_response(self) -> Response {
        self.data
            .with_body(|response| Template::with_meta(self.meta, response))
    }
}

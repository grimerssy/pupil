use std::{borrow::Cow, ops::Deref};

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
        locale: &LanguageIdentifier,
    ) -> Result<String, InternalError>
    where
        T: Serialize;
}

pub trait LocaleNegotiator {
    fn negotiate_locale(&self, sorted_preferences: Vec<LanguageIdentifier>) -> LanguageIdentifier;
}

#[derive(Clone, Debug)]
pub struct Template<T> {
    template_name: TemplateName,
    data: T,
}

#[derive(Clone, Debug)]
pub struct TemplateName(Cow<'static, str>);

impl<T> Template<T> {
    pub fn new(template_name: impl Into<TemplateName>, data: T) -> Self {
        Self {
            template_name: template_name.into(),
            data,
        }
    }
}

impl TemplateName {
    pub fn error() -> Self {
        Self::from("error.html")
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
    let locale = ctx.localizer.negotiate_locale(language_preferences);
    let html = match renderer.render_template(&template.template_name, template.data, &locale) {
        Ok(html) => html,
        Err(error) => {
            response = Template::new(TemplateName::error(), Rejection(error)).into_response();
            let template = response
                .extensions_mut()
                .remove::<Template<HttpResponse>>()
                .unwrap();
            renderer
                .render_template(&template.template_name, template.data, &locale)
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
            .with_body(|response| Template::new(self.template_name, response))
    }
}

impl<T> From<T> for TemplateName
where
    T: Into<Cow<'static, str>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Deref for TemplateName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

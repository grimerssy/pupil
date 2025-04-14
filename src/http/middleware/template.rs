use std::{borrow::Cow, sync::OnceLock};

use anyhow::{anyhow, Context};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;
use tera::Tera;

use super::error::error_response;

// TODO allocated renderer in app state
// - use extensions like in view
// - make a middleware layer with from_fn_with_state

static TEMPLATES: &str = "templates/**/*.html";

static RENDERER: OnceLock<Tera> = OnceLock::new();

fn renderer() -> crate::Result<&'static Tera> {
    match RENDERER.get() {
        Some(r) => Ok(r),
        None => {
            let tera = Tera::new(TEMPLATES).context("parse templates")?;
            RENDERER
                .set(tera)
                .map_err(|_| anyhow!("failed to set global renderer"))?;
            renderer()
        }
    }
}

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

#[tracing::instrument(skip_all, err(Debug))]
fn render_template<T>(template: Template<T>) -> crate::Result<Html<String>>
where
    T: Serialize,
{
    let renderer = renderer()?;
    #[cfg(debug_assertions)]
    let renderer = reload_templates(renderer)?;
    let context =
        tera::Context::from_serialize(template.data).context("serialize template context")?;
    let html = renderer
        .render(&template.meta.name, &context)
        .context("render template")?;
    Ok(Html(html))
}

#[cfg(debug_assertions)]
fn reload_templates(old: &'static Tera) -> crate::Result<Tera> {
    let mut renderer = old.clone();
    renderer.full_reload().context("reload templates")?;
    Ok(renderer)
}

impl<T> IntoResponse for Template<T>
where
    T: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        render_template(self)
            .map_err(Template::error)
            .into_response()
    }
}

// separate implementation is needed to ensure no recursion
// don't try to be smarter
impl IntoResponse for ErrorTemplate {
    fn into_response(self) -> Response {
        let into_template = |msg| {
            let t = Template {
                meta: self.meta,
                data: msg,
            };
            render_template(t).expect("render error template")
        };
        error_response(&self.data, into_template)
    }
}

use std::{borrow::Cow, sync::OnceLock};

use anyhow::{anyhow, Context};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;
use tera::Tera;

use super::error::ErrorResponse;

pub(super) const ERROR_TEMPLATE: &str = "error.html";

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
    name: Cow<'static, str>,
    data: T,
}

#[derive(Debug)]
pub struct ErrorTemplate(pub crate::Error);

pub type ResultTemplate<T> = Result<Template<T>, ErrorTemplate>;

impl<T> Template<T> {
    pub fn new<N>(name: N, data: T) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        Self {
            name: name.into(),
            data,
        }
    }
}

#[tracing::instrument(level = "trace", skip_all, err(Debug))]
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
        .render(&template.name, &context)
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
        render_template(self).map_err(ErrorTemplate).into_response()
    }
}

impl IntoResponse for ErrorTemplate {
    fn into_response(self) -> Response {
        let error = ErrorResponse::new(&self.0);
        let template = Template::new("error.html", error.message);
        // TODO when catch panic ensure no recursion
        // using Template as IntoResponse here would result in recursion
        let html = render_template(template).expect("render error template");
        (error.status_code, html).into_response()
    }
}

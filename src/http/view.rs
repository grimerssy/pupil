use std::{borrow::Cow, sync::OnceLock};

use anyhow::{anyhow, Context};
use axum::{
    extract::Request,
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Extension, Json,
};
use serde::Serialize;
use tera::Tera;

use super::response_type::{LazyResponseType, ResponseType};

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
pub struct View<T> {
    template_name: Cow<'static, str>,
    data: T,
}

impl<T> View<T> {
    pub fn new<N>(template_name: N, data: T) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        Self {
            template_name: template_name.into(),
            data,
        }
    }
}

pub async fn render_view(response_type: LazyResponseType, req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let Some(view) = response.extensions_mut().remove::<TypeErasedView>() else {
        return response;
    };
    match response_type.parse() {
        ResponseType::Json => Json(view.data).into_response(),
        ResponseType::Html => render_template(view).into_response(),
    }
}

#[tracing::instrument(level = "trace", skip_all, err(Debug))]
fn render_template(view: TypeErasedView) -> crate::Result<Html<String>> {
    let renderer = renderer()?;
    #[cfg(debug_assertions)]
    let renderer = {
        let mut renderer = renderer.clone();
        renderer.full_reload().context("reload templates")?;
        renderer
    };
    let context =
        tera::Context::from_serialize(view.data).context("serialize into template context")?;
    let html = renderer
        .render(&view.template_name, &context)
        .context("render template")?;
    Ok(Html(html))
}

struct TypeErasedView {
    template_name: Cow<'static, str>,
    data: Box<dyn erased_serde::Serialize + Send + Sync>,
}

impl Clone for TypeErasedView {
    fn clone(&self) -> Self {
        unreachable!("a view body may not be cloned")
    }
}

impl<T> IntoResponse for View<T>
where
    T: Serialize + Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let view = TypeErasedView {
            template_name: self.template_name,
            data: Box::new(self.data),
        };
        Extension(view).into_response()
    }
}

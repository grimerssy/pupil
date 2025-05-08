use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{context::AppContext, domain::error::InternalError, http::RenderTemplate};

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateConfig {
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct TemplateRenderer {
    tera: Arc<Tera>,
}

impl TemplateRenderer {
    pub fn new(config: TemplateConfig) -> anyhow::Result<Self> {
        let templates = config
            .path
            .to_str()
            .ok_or_else(|| anyhow!("template path contains invalid unicode"))?;
        let tera = Tera::new(templates)
            .map(Arc::new)
            .context("construct tera renderer")?;
        Ok(Self { tera })
    }
}

impl RenderTemplate for &AppContext {
    #[tracing::instrument(skip(self, data), err(Debug))]
    fn render_template<T>(self, template_name: &str, data: T) -> Result<String, InternalError>
    where
        T: Serialize,
    {
        render_template_with(&self.template_renderer, template_name, data)
    }
}

fn render_template_with<T>(
    renderer: &TemplateRenderer,
    template_name: &str,
    data: T,
) -> Result<String, InternalError>
where
    T: Serialize,
{
    let tera = renderer.tera.as_ref();
    #[cfg(debug_assertions)]
    let tera = reload_tera(tera).map_err(InternalError::from)?;
    let context = serde_json::to_value(data)
        .context("serialize template context")
        .map_err(InternalError::from)?;
    let mut tera_context = tera::Context::new();
    tera_context.insert("context", &context);
    let html = tera
        .render(template_name, &tera_context)
        .context("render template")
        .map_err(InternalError::from)?;
    Ok(html)
}

#[cfg(debug_assertions)]
fn reload_tera(renderer: &Tera) -> anyhow::Result<Tera> {
    let mut renderer = renderer.clone();
    renderer.full_reload().context("reload templates")?;
    Ok(renderer)
}

use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{context::AppContext, error::InternalError};

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateConfig {
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct TemplateRenderer {
    renderer: Arc<Tera>,
}

impl TemplateRenderer {
    pub fn new(config: TemplateConfig) -> anyhow::Result<Self> {
        let templates = config
            .path
            .to_str()
            .ok_or_else(|| anyhow!("template path contains invalid unicode"))?;
        let renderer = Arc::new(Tera::new(templates)?);
        Ok(Self { renderer })
    }
}

pub fn render_template_with<T>(
    ctx: &AppContext,
    template_name: &str,
    data: T,
) -> Result<String, InternalError>
where
    T: Serialize,
{
    render_template(&ctx.template_renderer.renderer, template_name, data)
}

#[tracing::instrument(skip(renderer, data), err(Debug))]
fn render_template<T>(
    renderer: &Tera,
    template_name: &str,
    data: T,
) -> Result<String, InternalError>
where
    T: Serialize,
{
    #[cfg(debug_assertions)]
    let renderer = reload_templates(renderer)?;
    let context = tera::Context::from_serialize(data).context("serialize template context")?;
    let html = renderer
        .render(template_name, &context)
        .context("render template")?;
    Ok(html)
}

#[cfg(debug_assertions)]
fn reload_templates(renderer: &Tera) -> Result<Tera, InternalError> {
    let mut renderer = renderer.clone();
    renderer.full_reload().context("reload templates")?;
    Ok(renderer)
}

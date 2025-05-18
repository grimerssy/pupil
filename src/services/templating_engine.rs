use std::path::PathBuf;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{domain::error::InternalError, http::TemplateRenderer};

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateConfig {
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct TemplatingEngine {
    tera: Tera,
}

impl TemplatingEngine {
    pub fn new(config: TemplateConfig) -> anyhow::Result<Self> {
        let templates = config
            .path
            .to_str()
            .ok_or_else(|| anyhow!("template path contains invalid unicode"))?;
        let tera = Tera::new(templates).context("construct tera renderer")?;
        Ok(Self { tera })
    }
}

impl TemplateRenderer for TemplatingEngine {
    #[tracing::instrument(skip(self, data), err(Debug))]
    fn render_template<T>(&self, template_name: &str, data: T) -> Result<String, InternalError>
    where
        T: Serialize,
    {
        render_template_with(self, template_name, data)
    }
}

fn render_template_with<T>(
    templating_engine: &TemplatingEngine,
    template_name: &str,
    data: T,
) -> Result<String, InternalError>
where
    T: Serialize,
{
    #[cfg(debug_assertions)]
    let templating_engine = reload_engine(templating_engine)?;
    let context = serde_json::to_value(data)
        .context("serialize template context")
        .map_err(InternalError::from)?;
    let mut tera_context = tera::Context::new();
    tera_context.insert("context", &context);
    let html = templating_engine
        .tera
        .render(template_name, &tera_context)
        .context("render template")
        .map_err(InternalError::from)?;
    Ok(html)
}

#[cfg(debug_assertions)]
fn reload_engine(templating_engine: &TemplatingEngine) -> Result<TemplatingEngine, InternalError> {
    let mut templating_engine = templating_engine.clone();
    templating_engine
        .tera
        .full_reload()
        .context("reload templates")?;
    Ok(templating_engine)
}

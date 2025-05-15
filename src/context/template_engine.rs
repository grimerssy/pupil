use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{context::AppContext, domain::error::InternalError, http::TemplateRenderer};

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateConfig {
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct TemplateEngine {
    tera: Arc<Tera>,
}

impl TemplateEngine {
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

    #[tracing::instrument(skip(self, data), err(Debug))]
    fn render_template<T>(&self, template_name: &str, data: T) -> Result<String, InternalError>
    where
        T: Serialize,
    {
        render_template_with(self, template_name, data)
    }
}

fn render_template_with<T>(
    renderer: &TemplateEngine,
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

impl TemplateRenderer for AppContext {
    fn render_template<T>(&self, template_name: &str, data: T) -> Result<String, InternalError>
    where
        T: Serialize,
    {
        self.template_engine.render_template(template_name, data)
    }
}

#[cfg(debug_assertions)]
fn reload_tera(renderer: &Tera) -> anyhow::Result<Tera> {
    let mut renderer = renderer.clone();
    renderer.full_reload().context("reload templates")?;
    Ok(renderer)
}

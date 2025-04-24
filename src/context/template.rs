use std::sync::Arc;

use anyhow::{anyhow, Context};
use serde::Serialize;
use tera::Tera;

use crate::{config::TemplateConfig, error::InternalError};

use super::AppContext;

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

impl AppContext {
    #[tracing::instrument(skip_all, err(Debug))]
    pub(crate) fn render_template<T>(&self, template_name: &str, data: T) -> Result<String, InternalError>
    where
        T: Serialize,
    {
        let renderer = &self.template_renderer.renderer;
        #[cfg(debug_assertions)]
        let renderer = reload_templates(renderer)?;
        let context = tera::Context::from_serialize(data).context("serialize template context")?;
        let html = renderer
            .render(template_name, &context)
            .context("render template")?;
        Ok(html)
    }
}

#[cfg(debug_assertions)]
fn reload_templates(renderer: &Tera) -> Result<Tera, InternalError> {
    let mut renderer = renderer.clone();
    renderer.full_reload().context("reload templates")?;
    Ok(renderer)
}

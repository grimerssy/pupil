use serde::Deserialize;

use crate::template::{TemplateConfig, TemplateRenderer};

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub templates: TemplateConfig,
}

#[derive(Clone)]
pub struct AppContext {
    pub template_renderer: TemplateRenderer,
}

impl AppContext {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let ctx = Self {
            template_renderer: TemplateRenderer::new(config.templates)?,
        };
        Ok(ctx)
    }
}

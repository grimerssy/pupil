mod template;

use template::TemplateRenderer;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppContext {
    template_renderer: TemplateRenderer,
}

impl AppContext {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let ctx = Self {
            template_renderer: TemplateRenderer::new(config.templates)?,
        };
        Ok(ctx)
    }
}

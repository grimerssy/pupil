use std::sync::Arc;

use anyhow::Context;
use serde::Serialize;
use tera::Tera;

use super::AppContext;

static TEMPLATES: &str = "templates/**/*.html";

#[derive(Clone)]
pub struct TemplateRenderer {
    renderer: Arc<Tera>,
}

impl TemplateRenderer {
    pub fn new() -> anyhow::Result<Self> {
        let renderer = Tera::new(TEMPLATES)?;
        let renderer = Arc::new(renderer);
        Ok(Self { renderer })
    }
}

impl AppContext {
    #[tracing::instrument(skip_all, err(Debug))]
    pub(crate) fn render_template<T>(&self, template_name: &str, data: T) -> crate::Result<String>
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
fn reload_templates(renderer: &Tera) -> crate::Result<Tera> {
    let mut renderer = renderer.clone();
    renderer.full_reload().context("reload templates")?;
    Ok(renderer)
}

use std::path::PathBuf;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{domain::error::InternalError, http::TemplateRenderer};

static LOCALIZE_FUNCTION: &str = "localize";

pub trait TemplateLocalizer: Clone {
    fn reload(&mut self) -> Result<(), InternalError>;

    fn into_function(self) -> impl tera::Function;
}

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateConfig {
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct TemplatingEngine<L> {
    tera: Tera,
    localizer: L,
}

impl<L> TemplatingEngine<L> {
    pub fn new(config: TemplateConfig, localizer: L) -> anyhow::Result<Self>
    where
        L: TemplateLocalizer + 'static,
    {
        let templates = config
            .path
            .to_str()
            .ok_or_else(|| anyhow!("template path contains invalid unicode"))?;
        let mut tera = Tera::new(templates).context("construct tera renderer")?;
        tera.register_function(LOCALIZE_FUNCTION, localizer.clone().into_function());
        Ok(Self { tera, localizer })
    }
}

impl<L> TemplateRenderer for TemplatingEngine<L>
where
    L: TemplateLocalizer + 'static,
{
    #[tracing::instrument(skip(self, data), err(Debug))]
    fn render_template<T>(&self, template_name: &str, data: T) -> Result<String, InternalError>
    where
        T: Serialize,
    {
        render_template_with(self, template_name, data)
    }
}

fn render_template_with<T, L>(
    templating_engine: &TemplatingEngine<L>,
    template_name: &str,
    data: T,
) -> Result<String, InternalError>
where
    T: Serialize,
    L: TemplateLocalizer + 'static,
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
fn reload_engine<L>(
    templating_engine: &TemplatingEngine<L>,
) -> Result<TemplatingEngine<L>, InternalError>
where
    L: TemplateLocalizer + 'static,
{
    let mut templating_engine = templating_engine.clone();
    templating_engine
        .tera
        .full_reload()
        .context("reload templates")?;
    templating_engine.localizer.reload()?;
    templating_engine.tera.register_function(
        LOCALIZE_FUNCTION,
        templating_engine.localizer.clone().into_function(),
    );
    Ok(templating_engine)
}

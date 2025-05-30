use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tera::Tera;
use unic_langid::LanguageIdentifier;

use crate::{app::localization::LocalizedError, http::TemplateRenderer};

static LOCALIZE_FUNCTION: &str = "localize";

static LOCALIZE_ERROR_FUNCTION: &str = "localize_error";

static ERROR: &str = "error";

static KEY: &str = "key";

pub trait TemplateLocalizer: Clone {
    fn reload(&mut self) -> crate::Result<()>;

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
        tera.register_function(
            LOCALIZE_ERROR_FUNCTION,
            localize_error(localizer.clone().into_function()),
        );
        Ok(Self { tera, localizer })
    }
}

impl<L> TemplateRenderer for TemplatingEngine<L>
where
    L: TemplateLocalizer + 'static,
{
    #[tracing::instrument(skip(self, data), err(Debug))]
    fn render_template<T>(
        &self,
        template_name: &str,
        data: T,
        locale: &LanguageIdentifier,
    ) -> crate::Result<String>
    where
        T: Serialize,
    {
        render_template(self, template_name, data, locale)
    }
}

fn render_template<T, L>(
    templating_engine: &TemplatingEngine<L>,
    template_name: &str,
    data: T,
    locale: &LanguageIdentifier,
) -> crate::Result<String>
where
    T: Serialize,
    L: TemplateLocalizer + 'static,
{
    #[cfg(debug_assertions)]
    let templating_engine = reload_engine(templating_engine)?;
    let context = serde_json::to_value(data).context("serialize template context")?;
    let mut tera_context = tera::Context::new();
    tera_context.insert("context", &context);
    tera_context.insert("locale", locale);
    let html = templating_engine
        .tera
        .render(template_name, &tera_context)
        .context("render template")?;
    Ok(html)
}

fn localize_error(localize: impl tera::Function) -> impl tera::Function {
    move |args: &HashMap<_, _>| {
        let mut args = args.clone();
        let error = args
            .remove(ERROR)
            .ok_or_else(|| tera::Error::msg(format!("missing `{ERROR}` argument")))
            .and_then(|json| LocalizedError::deserialize(json).map_err(tera::Error::msg))?;
        let error_key = core::iter::once("error")
            .chain(error.error_code().split('_'))
            .map(|word| word.to_lowercase())
            .collect::<Vec<_>>()
            .join("-");
        args.insert(KEY.to_owned(), serde_json::to_value(&error_key).unwrap());
        for (key, value) in error.args() {
            args.insert(key.to_owned(), serde_json::to_value(value).unwrap());
        }
        localize.call(&args)
    }
}

#[cfg(debug_assertions)]
fn reload_engine<L>(templating_engine: &TemplatingEngine<L>) -> crate::Result<TemplatingEngine<L>>
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
    templating_engine.tera.register_function(
        LOCALIZE_ERROR_FUNCTION,
        localize_error(templating_engine.localizer.clone().into_function()),
    );
    Ok(templating_engine)
}

use database::{Database, DatabaseConfig};
use hasher::{Hasher, HasherConfig};
use localizer::{I18nConfig, Localizer};
use serde::Deserialize;
use template::{TemplateConfig, TemplateRenderer};

mod database;
mod hasher;
mod localizer;
mod template;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub i18n: I18nConfig,
    pub database: DatabaseConfig,
    pub hasher: HasherConfig,
    pub templates: TemplateConfig,
}

#[derive(Clone)]
pub struct AppContext {
    #[allow(unused)]
    localizer: Localizer,
    database: Database,
    hasher: Hasher,
    template_renderer: TemplateRenderer,
}

impl AppContext {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let ctx = Self {
            localizer: Localizer::new(config.i18n)?,
            database: Database::new(config.database),
            hasher: Hasher::new(config.hasher)?,
            template_renderer: TemplateRenderer::new(config.templates)?,
        };
        Ok(ctx)
    }
}

use database::{Database, DatabaseConfig};
use hasher::{Hasher, HasherConfig};
use serde::Deserialize;
use template::{TemplateConfig, TemplateRenderer};

pub mod database;
pub mod hasher;
pub mod template;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub hasher: HasherConfig,
    pub templates: TemplateConfig,
}

#[derive(Clone)]
pub struct AppContext {
    database: Database,
    hasher: Hasher,
    template_renderer: TemplateRenderer,
}

impl AppContext {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let ctx = Self {
            database: Database::new(config.database),
            hasher: Hasher::new(config.hasher)?,
            template_renderer: TemplateRenderer::new(config.templates)?,
        };
        Ok(ctx)
    }
}

use serde::Deserialize;

use crate::{
    database::{Database, DatabaseConfig},
    error::InternalError,
    template::{TemplateConfig, TemplateRenderer},
};

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub templates: TemplateConfig,
    pub database: DatabaseConfig,
}

#[derive(Clone)]
pub struct AppContext {
    pub template_renderer: TemplateRenderer,
    pub database: Database,
}

impl AppContext {
    pub fn new(config: AppConfig) -> Result<Self, InternalError> {
        let ctx = Self {
            template_renderer: TemplateRenderer::new(config.templates)?,
            database: Database::new(config.database),
        };
        Ok(ctx)
    }
}

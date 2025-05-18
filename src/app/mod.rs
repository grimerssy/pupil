use std::sync::Arc;

use serde::Deserialize;

use crate::services::{
    database::{Database, DatabaseConfig},
    hasher::{Hasher, HasherConfig},
    localizer::{I18nConfig, Localizer},
    templating_engine::{TemplateConfig, TemplatingEngine},
};

pub mod auth;
pub mod error;
pub mod validation;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub i18n: I18nConfig,
    pub database: DatabaseConfig,
    pub hasher: HasherConfig,
    pub templates: TemplateConfig,
}

#[derive(Clone)]
pub struct AppContext {
    pub localizer: Arc<Localizer>,
    pub database: Database,
    pub hasher: Hasher,
    pub templating_engine: Arc<TemplatingEngine<Arc<Localizer>>>,
}

impl AppContext {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let database = Database::new(config.database);
        let hasher = Hasher::new(config.hasher)?;
        let localizer = Arc::new(Localizer::new(config.i18n)?);
        let templating_engine =
            Arc::new(TemplatingEngine::new(config.templates, localizer.clone())?);
        Ok(Self {
            localizer,
            database,
            hasher,
            templating_engine,
        })
    }
}

use std::sync::Arc;

use serde::Deserialize;
use validation::ValidationErrors;

use crate::services::{
    database::{Database, DatabaseConfig},
    hasher::{Hasher, HasherConfig},
    id_encoder::{IdConfig, IdEncoder},
    localizer::{I18nConfig, Localizer},
    templating_engine::{TemplateConfig, TemplatingEngine},
    token_issuer::{JwtConfig, TokenIssuer},
};

pub mod auth;
pub mod localization;
pub mod validation;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub i18n: I18nConfig,
    pub database: DatabaseConfig,
    pub id: IdConfig,
    pub hasher: HasherConfig,
    pub jwt: JwtConfig,
    pub templates: TemplateConfig,
}

#[derive(Clone)]
pub struct AppContext {
    pub localizer: Arc<Localizer>,
    pub database: Database,
    pub id_encoder: Arc<IdEncoder>,
    pub hasher: Hasher,
    pub token_issuer: TokenIssuer,
    pub templating_engine: Arc<TemplatingEngine<Arc<Localizer>>>,
}

#[derive(Debug)]
pub enum AppError<E> {
    Validation(ValidationErrors),
    Logical(E),
}

impl AppContext {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let hasher = Hasher::new(config.hasher)?;
        let localizer = Arc::new(Localizer::new(config.i18n)?);
        let templating_engine =
            Arc::new(TemplatingEngine::new(config.templates, localizer.clone())?);
        let database = Database::new(config.database);
        let id_encoder = Arc::new(IdEncoder::new(config.id));
        let token_issuer = TokenIssuer::new(config.jwt);
        Ok(Self {
            localizer,
            database,
            id_encoder,
            hasher,
            token_issuer,
            templating_engine,
        })
    }
}

impl<E> From<E> for AppError<E> {
    fn from(value: E) -> Self {
        Self::Logical(value)
    }
}

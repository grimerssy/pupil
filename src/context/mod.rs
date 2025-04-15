use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppContext;

impl AppContext {
    pub fn new(_config: AppConfig) -> anyhow::Result<Self> {
        let ctx = Self;
        Ok(ctx)
    }
}

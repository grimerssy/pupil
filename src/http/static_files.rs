use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::AppContext;

const BUILD_DIR: &str = "dist";
const ASSET_DIR: &str = "assets";
const FAVICON: &str = "favicon.ico";

pub fn static_router() -> Router<AppContext> {
    let favicon = ServeFile::new(format!("{ASSET_DIR}/{FAVICON}"));
    let static_assets = ServeDir::new(BUILD_DIR).not_found_service(ServeDir::new(ASSET_DIR));
    Router::new()
        .nest_service(&format!("/{FAVICON}"), favicon)
        .nest_service("/static", static_assets)
}

use std::net::SocketAddr;

use anyhow::Context;
use axum::{routing::get, Router};
use tokio::net::TcpListener;

use crate::config::HttpConfig;

pub async fn serve(config: HttpConfig) -> anyhow::Result<()> {
    let router = router();
    let addr = SocketAddr::from((config.host, config.port));
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router.into_make_service())
        .await
        .context("start http server")
}

fn router() -> Router {
    Router::new().route("/", get(async || "Hello, World!"))
}

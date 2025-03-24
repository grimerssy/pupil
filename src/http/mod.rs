mod error;
mod response_type;
mod view;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{middleware, routing::get, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use view::{render_view, View};

use crate::config::HttpConfig;

pub async fn serve(config: HttpConfig) -> anyhow::Result<()> {
    let router = router();
    let addr = SocketAddr::from((config.host, config.port));
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router.into_make_service())
        .await
        .context("start http server")
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Index {}

fn router() -> Router {
    Router::new()
        .route("/", get(View::new("index.html", Index {})))
        .layer(middleware::from_fn(render_view))
}

mod middleware;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{extract::Query, routing::get, Router};
use middleware::view::{render_view, ErrorView, ResultView, View};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::config::HttpConfig;

pub async fn serve(config: HttpConfig) -> anyhow::Result<()> {
    let router = router();
    let addr = SocketAddr::from((config.host, config.port));
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router.into_make_service())
        .await
        .context("start http server")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Index {
    name: Option<String>,
}

fn router() -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("dist"))
        .route("/", get(index))
        .layer(axum::middleware::from_fn(render_view))
}

#[tracing::instrument(level = "trace", ret(level = "debug"))]
async fn index(Query(mut idx): Query<Index>) -> View<Index> {
    idx.name.get_or_insert_with(|| "World".into());
    View::new("index.html", idx)
}

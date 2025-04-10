mod middleware;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{extract::Query, routing::get, Router};
use middleware::{
    error::{handle_not_found, handle_panic},
    view::{render_view, ResultView, View},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::{catch_panic::CatchPanicLayer, services::ServeDir};

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
        .fallback(handle_not_found)
        .layer(CatchPanicLayer::custom(handle_panic))
        .layer(axum::middleware::from_fn(render_view))
}

#[tracing::instrument(level = "trace", ret(level = "debug"), err(Debug))]
async fn index(Query(mut idx): Query<Index>) -> ResultView<Index> {
    idx.name.get_or_insert_with(|| "World".into());
    // suppress unused
    let idx: crate::Result<Index> = Ok(idx);
    let idx = idx.map_err(View::error)?;
    let view = View::new("index.html", idx);
    Ok(view)
}

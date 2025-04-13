mod middleware;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{routing::get, Router};
use middleware::{
    error::{handle_not_found, handle_panic},
    query::Query,
    view::{render_view, ResultView, View},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::{catch_panic::CatchPanicLayer, services::ServeDir, trace::TraceLayer};

pub(crate) use middleware::error::ExtractionError;

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
    name: String,
}

fn router() -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("dist"))
        .route("/", get(index))
        .fallback(handle_not_found)
        .layer(CatchPanicLayer::custom(handle_panic))
        .layer(axum::middleware::from_fn(render_view))
        .layer(TraceLayer::new_for_http())
}

#[tracing::instrument(skip_all)]
async fn index(idx: Query<Index>) -> ResultView<Index> {
    let idx = idx.consume().map_err(View::error)?;
    let idx = process_index(idx).map_err(View::error)?;
    let view = View::new("index.html", idx);
    Ok(view)
}

#[tracing::instrument(ret, err(Debug))]
fn process_index(idx: Index) -> crate::Result<Index> {
    Ok(idx)
}

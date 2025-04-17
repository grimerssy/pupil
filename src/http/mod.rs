mod middleware;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{routing::get, Router};
use middleware::{
    error::{handle_not_found, handle_panic},
    query::Query,
    template::render_template,
    view::{render_view, ResultView, View},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::{catch_panic::CatchPanicLayer, services::ServeDir, trace::TraceLayer};

pub(crate) use middleware::error::ExtractionError;

use crate::{config::HttpConfig, context::AppContext};

pub async fn serve(config: HttpConfig, ctx: AppContext) -> anyhow::Result<()> {
    let router = router(ctx);
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

fn router(ctx: AppContext) -> Router {
    let handle_panic = CatchPanicLayer::custom(handle_panic);
    let render_view = axum::middleware::from_fn(render_view);
    let render_template = axum::middleware::from_fn_with_state(ctx.clone(), render_template);
    let trace = TraceLayer::new_for_http();
    Router::new()
        .nest_service("/static", ServeDir::new("dist"))
        .route("/", get(index))
        .fallback(handle_not_found)
        .layer(handle_panic)
        .layer(render_view)
        .layer(render_template)
        .layer(trace)
        .with_state(ctx)
}

#[tracing::instrument(skip_all)]
async fn index(idx: Query<Index>) -> ResultView<Index> {
    let idx = idx.consume().map_err(View::error)?;
    let view = View::new("index.html", idx);
    Ok(view)
}

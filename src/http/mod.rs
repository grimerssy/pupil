mod middleware;

use std::{convert::Infallible, net::SocketAddr};

use anyhow::Context;
use axum::{extract::Query, routing::get, Router};
use middleware::{
    view::{ResultView, View},
    RouterExt,
};
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::context::AppContext;

#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    pub host: [u8; 4],
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

pub async fn serve(config: HttpConfig, ctx: AppContext) -> anyhow::Result<()> {
    let router = root().with_middleware(ctx.clone()).with_state(ctx);
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

fn root() -> Router<AppContext> {
    Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("dist"))
}

#[tracing::instrument(skip_all)]
async fn index(Query(idx): Query<Index>) -> ResultView<Index, Infallible> {
    let view = View::new("index.html", idx);
    Ok(view)
}

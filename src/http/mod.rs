use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Context;
use axum::{routing::get, Router};
use tokio::net::TcpListener;

pub async fn serve() -> anyhow::Result<()> {
    let router = router();
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 8080));
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router.into_make_service())
        .await
        .context("start http server")
}

fn router() -> Router {
    Router::new().route("/", get(async || "Hello, World!"))
}

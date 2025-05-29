pub use middleware::template::{LocaleNegotiator, TemplateRenderer};

use auth::auth_routes;
use secrecy::{zeroize::Zeroize, ExposeSecret, SecretBox};
use static_files::static_router;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{routing::get, Router};
use middleware::{template::Template, RouterExt};
use serde::{Deserialize, Serialize, Serializer};
use serde_aux::field_attributes::deserialize_number_from_string;
use tokio::net::TcpListener;

use crate::app::AppContext;

mod error;
mod middleware;

mod auth;
mod static_files;

#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    pub host: [u8; 4],
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

pub async fn serve_http(config: HttpConfig, ctx: AppContext) -> anyhow::Result<()> {
    let addr = SocketAddr::from((config.host, config.port));
    let listener = TcpListener::bind(addr).await?;
    let router = root_router(ctx);
    axum::serve(listener, router.into_make_service())
        .await
        .context("start http server")
}

fn root_router(ctx: AppContext) -> Router {
    Router::new()
        .route("/", get(index))
        .nest("/auth", auth_routes())
        .with_middleware(ctx.clone())
        .with_state(ctx)
        .merge(static_router())
}

async fn index() -> Template<()> {
    Template::new("index.html", ())
}

fn serialize_secret<T, S>(secret: &SecretBox<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize + Zeroize + ?Sized,
    S: Serializer,
{
    secret.expose_secret().serialize(serializer)
}

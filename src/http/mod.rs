use grades::grades_routes;
use keys::keys_routes;
pub use middleware::template::{LocaleNegotiator, TemplateRenderer};

use auth::auth_routes;
use performance::performance_routes;
use secrecy::{zeroize::Zeroize, ExposeSecret, SecretBox};
use static_files::static_router;
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};

use std::net::SocketAddr;

use anyhow::Context;
use axum::{response::Html, routing::get, Router};
use middleware::{not_found::not_found_view, panic::catch_panic, template::Template, RouterExt};
use serde::{Deserialize, Serialize, Serializer};
use serde_aux::field_attributes::deserialize_number_from_string;
use tokio::net::TcpListener;

use crate::app::AppContext;

mod error;
mod middleware;

mod auth;
mod grades;
mod keys;
mod performance;

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
    let router = root_router()
        .fallback(not_found_view)
        .layer(CatchPanicLayer::custom(catch_panic))
        .with_renderers(ctx.clone())
        .layer(TraceLayer::new_for_http())
        .with_state(ctx)
        .merge(static_router());
    axum::serve(listener, router.into_make_service())
        .await
        .context("start http server")
}

fn root_router() -> Router<AppContext> {
    Router::new()
        .route("/", get(homepage))
        .route("/empty", get(async || Html("")))
        .nest("/auth", auth_routes())
        .nest("/grades", grades_routes())
        .nest("/keys", keys_routes())
        .nest("/performance", performance_routes())
}

async fn homepage() -> Template<()> {
    Template::new("index.html", ())
}

fn serialize_secret<T, S>(secret: &SecretBox<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize + Zeroize + ?Sized,
    S: Serializer,
{
    secret.expose_secret().serialize(serializer)
}

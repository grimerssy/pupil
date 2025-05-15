pub mod template;
pub mod view;

mod localization;
mod not_found;
mod panic;

pub use localization::{DefaultLanguage, LookupLanguage};

use localization::{assert_valid_locale, redirect_to_default_locale};

use axum::{middleware, routing::get, Router};
use not_found::not_found_view;
use panic::catch_panic;
use template::render_template;
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};
use view::render_view;

use crate::context::AppContext;

pub trait RouterExt {
    fn with_middleware(self, ctx: AppContext) -> Self;
}

impl RouterExt for Router<AppContext> {
    fn with_middleware(self, ctx: AppContext) -> Self {
        Router::new()
            .nest("/{locale}/", self)
            .fallback(not_found_view)
            .layer(CatchPanicLayer::custom(catch_panic))
            .layer(middleware::from_fn(render_view))
            .layer(middleware::from_fn_with_state(ctx.clone(), render_template))
            .layer(middleware::from_fn_with_state(ctx, assert_valid_locale))
            .route("/", get(redirect_to_default_locale))
            .layer(TraceLayer::new_for_http())
    }
}

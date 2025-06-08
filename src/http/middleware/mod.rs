pub mod auth;
pub mod json;
pub mod not_found;
pub mod panic;
pub mod template;
pub mod view;

mod response;

use axum::{middleware, Router};
use template::render_template;
use view::render_view;

use crate::app::AppContext;

pub trait RouterExt {
    fn with_renderers(self, ctx: AppContext) -> Self;
}

impl RouterExt for Router<AppContext> {
    fn with_renderers(self, ctx: AppContext) -> Self {
        self.layer(middleware::from_fn(render_view))
            .layer(middleware::from_fn_with_state(ctx.clone(), render_template))
    }
}

pub mod error;
pub mod response_type;
pub mod template;
pub mod view;

use axum::{middleware, Router};
use error::{handle_not_found, handle_panic};
use template::handle_render_template;
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};
use view::handle_render_view;

use crate::context::AppContext;

pub trait RouterExt: private::Sealed {
    fn with_middleware(self, ctx: AppContext) -> Self;
}

impl<S> RouterExt for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn with_middleware(self, ctx: AppContext) -> Self {
        self.fallback(handle_not_found)
            .layer(CatchPanicLayer::custom(handle_panic))
            .layer(middleware::from_fn(handle_render_view))
            .layer(middleware::from_fn_with_state(ctx, handle_render_template))
            .layer(TraceLayer::new_for_http())
    }
}

mod private {
    pub trait Sealed {}
    impl<S> Sealed for axum::Router<S> {}
}

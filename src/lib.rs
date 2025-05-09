pub mod prelude {
    pub use super::{
        config::Config, context::AppContext, http::serve_http, telemetry::init_telemetry,
    };
}

mod app;
mod config;
mod context;
mod domain;
mod http;
mod telemetry;

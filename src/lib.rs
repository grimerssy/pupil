pub mod prelude {
    pub use super::{app::AppContext, config::Config, http::serve_http, telemetry::init_telemetry};
}

mod app;
mod config;
mod domain;
mod error;
mod http;
mod services;
mod telemetry;

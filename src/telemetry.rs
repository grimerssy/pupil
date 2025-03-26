use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .init()
}

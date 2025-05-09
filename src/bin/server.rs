use pupil::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_telemetry();
    let config = Config::init()?;
    let ctx = AppContext::new(config.app)?;
    serve_http(config.http, ctx).await
}

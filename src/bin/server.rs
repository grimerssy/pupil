#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pupil::telemetry::init();
    let config = pupil::config::Config::init()?;
    let ctx = pupil::context::AppContext::new(config.app)?;
    pupil::http::serve(config.http, ctx).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pupil::telemetry::init();
    let config = pupil::config::Config::init()?;
    pupil::http::serve(config.http).await
}

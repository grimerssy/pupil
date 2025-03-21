#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = pupil::config::Config::init()?;
    pupil::http::serve(config.http).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pupil::http::serve().await
}

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Local dev reads backend/.env: in Docker the variables come from compose,
    // so a missing file is expected, not an error.
    dotenvy::dotenv().ok();
    sensecare_api::telemetry::init();

    let config = sensecare_api::config::Config::from_env().context("invalid configuration")?;
    sensecare_api::run(config).await
}

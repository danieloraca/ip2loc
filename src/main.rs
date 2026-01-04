use iploc::{AppConfig, app_with_config};
use std::time::Duration;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("server error: {e}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env_with_cache(Duration::from_secs(3600));
    let app = app_with_config(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

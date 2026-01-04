use iploc::{AppConfig, app_with_config};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("server error: {e}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Build configuration from environment variables and construct the app
    let config = AppConfig::from_env();
    let app = app_with_config(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

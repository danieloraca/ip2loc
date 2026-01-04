use iploc::{AppState, app_with_state};
use std::time::Duration;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("server error: {e}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::from_env_with_cache(Duration::from_secs(3600));
    let app = app_with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

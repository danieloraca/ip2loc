use axum::{
    Router,
    extract::Query,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("server error: {e}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/geo", get(geo));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn geo(Query(q): Query<HashMap<String, String>>) -> Response {
    let api_key = match std::env::var("IP2LOCATIONIO_KEY") {
        Ok(k) => k,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "missing IP2LOCATIONIO_KEY",
            )
                .into_response();
        }
    };

    let ip = match q.get("ip").and_then(|s| s.parse::<std::net::IpAddr>().ok()) {
        Some(ip) => ip,
        None => return (StatusCode::BAD_REQUEST, "invalid ip").into_response(),
    };

    let url = format!("https://api.ip2location.io/?key={api_key}&ip={ip}");

    let resp = match reqwest::Client::new().get(url).send().await {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_GATEWAY, "provider request failed").into_response(),
    };

    let resp = match resp.error_for_status() {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_GATEWAY, "provider returned error").into_response(),
    };

    let body = match resp.text().await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_GATEWAY, "provider read failed").into_response(),
    };

    let mut res = (StatusCode::OK, body).into_response();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    res
}

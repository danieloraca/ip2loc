use axum::{
    Router,
    extract::{Extension, Query},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use std::{collections::HashMap, net::IpAddr, sync::Arc};

#[derive(Clone)]
pub struct AppConfig {
    pub api_key: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let api_key = std::env::var("IP2LOCATIONIO_KEY").ok();
        AppConfig { api_key }
    }
}

pub fn app_with_config(config: AppConfig) -> Router {
    Router::new()
        .route("/geo", get(geo))
        .layer(Extension(Arc::new(config)))
}

pub fn app() -> Router {
    app_with_config(AppConfig::from_env())
}

async fn geo(
    Query(q): Query<HashMap<String, String>>,
    Extension(config): Extension<Arc<AppConfig>>,
) -> Response {
    // Decide on API key from injected configuration
    let api_key = match &config.api_key {
        Some(k) => k.clone(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "missing IP2LOCATIONIO_KEY",
            )
                .into_response();
        }
    };

    // Parse the IP parameter
    let ip = match q.get("ip").and_then(|s| s.parse::<IpAddr>().ok()) {
        Some(ip) => ip,
        None => return (StatusCode::BAD_REQUEST, "invalid ip").into_response(),
    };

    // Prepare URL for provider
    let url = format!("https://api.ip2location.io/?key={api_key}&ip={ip}");

    // Perform request to provider
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

use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub api_key: Option<Arc<str>>,
    pub cache_ttl: Duration,
    cache: Option<IpCache>,
    client: reqwest::Client,
}

impl AppState {
    pub fn from_env() -> Self {
        let api_key = std::env::var("IP2LOCATIONIO_KEY")
            .ok()
            .map(|s| Arc::from(s.as_str()));
        AppState {
            api_key,
            cache_ttl: Duration::from_secs(0),
            cache: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env_with_cache(ttl: Duration) -> Self {
        let api_key = std::env::var("IP2LOCATIONIO_KEY")
            .ok()
            .map(|s| Arc::from(s.as_str()));
        let cache = if ttl > Duration::from_secs(0) {
            Some(build_cache())
        } else {
            None
        };

        AppState {
            api_key,
            cache_ttl: ttl,
            cache,
            client: reqwest::Client::new(),
        }
    }

    pub fn new(api_key: Option<Arc<str>>, cache_ttl: Duration) -> Self {
        let cache = if cache_ttl > Duration::from_secs(0) {
            Some(build_cache())
        } else {
            None
        };

        AppState {
            api_key,
            cache_ttl,
            cache,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Clone)]
struct CacheEntry {
    body: String,
    expires_at: Instant,
}

type IpCache = Arc<Mutex<HashMap<IpAddr, CacheEntry>>>;

fn build_cache() -> IpCache {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn app_with_state(state: AppState) -> Router {
    let shared_state = Arc::new(state);

    Router::new()
        .route("/geo", get(geo))
        .with_state(shared_state)
}

pub fn app() -> Router {
    app_with_state(AppState::from_env())
}

async fn geo(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let api_key = match &state.api_key {
        Some(k) => k,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "missing IP2LOCATIONIO_KEY",
            )
                .into_response();
        }
    };

    let ip = match q.get("ip").and_then(|s| s.parse::<IpAddr>().ok()) {
        Some(ip) => ip,
        None => return (StatusCode::BAD_REQUEST, "invalid ip").into_response(),
    };

    if let Some(cache) = &state.cache {
        let mut cache_guard = cache.lock().await;
        if let Some(entry) = cache_guard.get(&ip) {
            if entry.expires_at > Instant::now() {
                let mut res = (StatusCode::OK, entry.body.clone()).into_response();
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                return res;
            } else {
                cache_guard.remove(&ip);
            }
        }
        drop(cache_guard);
    }

    let url = format!("https://api.ip2location.io/?key={api_key}&ip={ip}");

    let resp = match state.client.get(url).send().await {
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

    if let Some(cache) = &state.cache {
        if state.cache_ttl > Duration::from_secs(0) {
            let mut cache_guard = cache.lock().await;
            cache_guard.insert(
                ip,
                CacheEntry {
                    body: body.clone(),
                    expires_at: Instant::now() + state.cache_ttl,
                },
            );
        }
    }

    let mut res = (StatusCode::OK, body).into_response();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    res
}

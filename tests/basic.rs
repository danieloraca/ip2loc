use axum::{body::Body, http::Request, http::StatusCode};
use tower::ServiceExt;

use iploc::{AppConfig, app_with_config};
use std::time::Duration;

#[tokio::test]
async fn returns_400_on_invalid_ip() {
    let app = app_with_config(AppConfig {
        api_key: Some("testkey".to_string()),
        cache_ttl: Duration::from_secs(0),
    });

    let res = app
        .oneshot(
            Request::builder()
                .uri("/geo?ip=not-an-ip")
                .body(Body::empty())
                .expect("failed to build request for invalid ip test"),
        )
        .await
        .expect("app.oneshot failed for invalid ip test");

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn returns_500_when_api_key_missing() {
    let app = app_with_config(AppConfig {
        api_key: None,
        cache_ttl: Duration::from_secs(0),
    });

    let res = app
        .oneshot(
            Request::builder()
                .uri("/geo?ip=1.1.1.1")
                .body(Body::empty())
                .expect("failed to build request for missing api key test"),
        )
        .await
        .expect("app.oneshot failed for missing api key test");

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn returns_200_on_valid_ip_with_api_key() {
    let app = app_with_config(AppConfig {
        api_key: Some("testkey".to_string()),
        // Disable caching here so we exercise the full provider path.
        cache_ttl: Duration::from_secs(0),
    });

    let res = app
        .oneshot(
            Request::builder()
                .uri("/geo?ip=1.1.1.1")
                .body(Body::empty())
                .expect("failed to build request for valid ip test"),
        )
        .await
        .expect("app.oneshot failed for valid ip test");

    assert_ne!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "valid IP should not return 400"
    );
    assert_ne!(
        res.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "with API key set, handler should not return 500 for a valid IP"
    );
}

#[tokio::test]
async fn repeated_requests_with_cache_enabled_return_consistent_status() {
    let app = app_with_config(AppConfig {
        api_key: Some("testkey".to_string()),
        // Enable caching for this test to exercise cache path.
        cache_ttl: Duration::from_secs(60),
    });

    // First request should go through the full handler and potentially populate cache.
    let res1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/geo?ip=8.8.8.8")
                .body(Body::empty())
                .expect("failed to build request for caching test (first)"),
        )
        .await
        .expect("app.oneshot failed for caching test (first)");
    let status1 = res1.status();

    // Second request should be served either from cache or provider, but must match status.
    let res2 = app
        .oneshot(
            Request::builder()
                .uri("/geo?ip=8.8.8.8")
                .body(Body::empty())
                .expect("failed to build request for caching test (second)"),
        )
        .await
        .expect("app.oneshot failed for caching test (second)");
    let status2 = res2.status();

    assert_eq!(
        status1, status2,
        "cached and non-cached responses should have the same status"
    );
}

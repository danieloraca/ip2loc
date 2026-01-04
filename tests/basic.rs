use axum::{body, body::Body, http::Request, http::StatusCode};
use tower::ServiceExt;

use iploc::{AppState, app_with_state};
use std::{sync::Arc, time::Duration};

#[tokio::test]
async fn returns_400_on_invalid_ip() {
    let app = app_with_state(AppState::new(
        Some(Arc::from("testkey")),
        Duration::from_secs(0),
    ));

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
async fn rejects_loopback_ip_with_400() {
    let app = app_with_state(AppState::new(
        Some(Arc::from("testkey")),
        Duration::from_secs(0),
    ));

    let res = app
        .oneshot(
            Request::builder()
                .uri("/geo?ip=127.0.0.1")
                .body(Body::empty())
                .expect("failed to build request for loopback ip test"),
        )
        .await
        .expect("app.oneshot failed for loopback ip test");

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_private_ip_with_400() {
    let app = app_with_state(AppState::new(
        Some(Arc::from("testkey")),
        Duration::from_secs(0),
    ));

    let res = app
        .oneshot(
            Request::builder()
                .uri("/geo?ip=192.168.1.1")
                .body(Body::empty())
                .expect("failed to build request for private ip test"),
        )
        .await
        .expect("app.oneshot failed for private ip test");

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn returns_500_when_api_key_missing() {
    let app = app_with_state(AppState::new(None, Duration::from_secs(0)));

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
async fn returns_non_error_on_valid_ip_with_api_key() {
    let app = app_with_state(AppState::new(
        Some(Arc::from("testkey")),
        // Disable caching here so we exercise the full provider path.
        Duration::from_secs(0),
    ));

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
    let app = app_with_state(AppState::new(
        Some(Arc::from("testkey")),
        Duration::from_secs(60),
    ));

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

#[tokio::test]
async fn cached_responses_are_annotated_when_flag_enabled() {
    // Build state with caching enabled and annotation flag turned on.
    let app = app_with_state(AppState::new_with_cached_flag(
        Some(Arc::from("testkey")),
        Duration::from_secs(60),
    ));

    let res1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/geo?ip=1.1.1.1")
                .body(Body::empty())
                .expect("failed to build request for cached flag test (first)"),
        )
        .await
        .expect("app.oneshot failed for cached flag test (first)");
    let status1 = res1.status();

    let body1_bytes: axum::body::Bytes = body::to_bytes(res1.into_body(), usize::MAX)
        .await
        .expect("failed to read first response body");
    let body1 =
        String::from_utf8(body1_bytes.to_vec()).expect("first response body was not valid UTF-8");

    let res2 = app
        .oneshot(
            Request::builder()
                .uri("/geo?ip=1.1.1.1")
                .body(Body::empty())
                .expect("failed to build request for cached flag test (second)"),
        )
        .await
        .expect("app.oneshot failed for cached flag test (second)");
    let status2 = res2.status();

    let body2_bytes: axum::body::Bytes = body::to_bytes(res2.into_body(), usize::MAX)
        .await
        .expect("failed to read second response body");
    let body2 =
        String::from_utf8(body2_bytes.to_vec()).expect("second response body was not valid UTF-8");

    if status1.is_success() && status2.is_success() && status1 == status2 {
        assert!(
            !body1.contains("\"cached\":true"),
            "first (uncached) response should not contain cached flag"
        );
        assert!(
            body2.contains("\"cached\":true"),
            "second (cached) response should contain cached flag"
        );
    }
}

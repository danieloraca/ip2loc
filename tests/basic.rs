use axum::{body::Body, http::Request, http::StatusCode};
use tower::ServiceExt;

use iploc::{AppConfig, app_with_config};

#[tokio::test]
async fn returns_400_on_invalid_ip() {
    let app = app_with_config(AppConfig {
        api_key: Some("testkey".to_string()),
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
    let app = app_with_config(AppConfig { api_key: None });

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

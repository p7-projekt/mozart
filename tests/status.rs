use axum::{
    body::{Body, HttpBody},
    http::{request::Builder, Method, StatusCode},
};
use mozart::app;
use tower::ServiceExt;

#[tokio::test]
async fn invalid_http_method() {
    let mozart = app();
    let expected_status_code = StatusCode::METHOD_NOT_ALLOWED;
    let request = Builder::new()
        .method(Method::POST)
        .uri("/status")
        .body(Body::empty())
        .expect("failed to build request");

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to await oneshot");

    assert_eq!(actual.status(), expected_status_code);
}

#[tokio::test]
async fn body_content_does_not_affect_request() {
    let mozart = app();
    let expected_status_code = StatusCode::OK;
    let request = Builder::new()
        .method(Method::GET)
        .uri("/status")
        .body(Body::from("Hello, world!"))
        .expect("failed to build request");

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to await oneshot");

    assert_eq!(actual.status(), expected_status_code);
}

#[tokio::test]
async fn valid() {
    let mozart = app();
    let expected_status_code = StatusCode::OK;
    let request = Builder::new()
        .method(Method::GET)
        .uri("/status")
        .body(Body::empty())
        .expect("failed to build request");

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to await oneshot");

    assert_eq!(actual.status(), expected_status_code);
    assert!(actual.body().is_end_stream());
}

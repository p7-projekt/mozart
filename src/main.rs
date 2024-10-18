use axum::{
    http::StatusCode,
    routing::{get, post},
    serve, Json, Router,
};
use error::SubmissionError;
use model::{Submission, TestResult};
use response::SubmissionResult;
use runner::TestRunner;
use std::{fs, path::PathBuf};
use tokio::net::TcpListener;
use tracing::{debug, error, info, span, Level};
use uuid::Uuid;

mod error;
mod log;
mod model;
mod response;
mod runner;
mod timeout;

/// The parent directory of all test runner jobs.
const PARENT_DIR: &str = "/tmp";

/// We need to initilize the logger before multithreading happens
/// otherwise local time offset cannot be determined.
///
/// As a result we initialise the logger inside a synchronous main function,
/// before calling the async tokio main function
fn main() {
    log::init();

    start();
}

fn app() -> Router {
    Router::new()
        .route("/submit", post(submit))
        .route("/status", get(status))
}

#[tokio::main]
async fn start() {
    let mozart = app();
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to localhost:8080");
    serve(listener, mozart)
        .await
        .expect("failed to start mozart");
}

async fn status() -> StatusCode {
    info!("performed status check");
    StatusCode::OK
}

async fn submit(Json(submission): Json<Submission>) -> SubmissionResult {
    let uuid = Uuid::new_v4();
    let span = span!(Level::TRACE, "", %uuid);
    let _enter = span.enter();

    debug!(?submission);

    let temp_dir = PathBuf::from(format!("{}/{}", PARENT_DIR, uuid));

    if let Err(err) = fs::create_dir(temp_dir.as_path()) {
        error!("could not create temporary working directory: {}", err);
        return SubmissionResult::from(SubmissionError::Internal);
    }

    let runner = TestRunner::new(temp_dir.clone());

    info!("checking submission");
    let response = match runner.check(submission).await {
        Ok(test_case_results) => {
            debug!(?test_case_results);
            if test_case_results
                .iter()
                .all(|tc| tc.test_result == TestResult::Pass)
            {
                info!("passed all test cases");
                SubmissionResult::Pass
            } else {
                info!("did not pass all test cases");
                SubmissionResult::Failure(test_case_results)
            }
        }
        Err(err) => SubmissionResult::from(err),
    };

    if let Err(err) = fs::remove_dir_all(temp_dir.as_path()) {
        error!("could not delete temporary working directory: {}", err);
        return SubmissionResult::from(SubmissionError::Internal);
    }

    response
}

#[cfg(test)]
mod endpoints {
    mod status {
        use crate::app;
        use axum::{
            body::{Body, HttpBody},
            http::{request::Builder, Method, StatusCode},
        };
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
    }
}

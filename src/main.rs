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
use time::{format_description::well_known::Rfc3339, UtcOffset};
use tokio::net::TcpListener;
use tracing::{error, span, Level};
use tracing_subscriber::fmt::time::OffsetTime;
use uuid::Uuid;

mod error;
mod model;
mod response;
mod runner;
mod timeout;

/// The parent directory of all test runner jobs.
const PARENT_DIR: &str = "/tmp";

fn app() -> Router {
    Router::new()
        .route("/submit", post(submit))
        .route("/status", get(status))
}

#[tokio::main]
async fn main() {
    let offset = UtcOffset::from_hms(2, 0, 0).expect("failed to create offset");
    let time = OffsetTime::new(offset, Rfc3339);
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_timer(time)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_target(false)
        .try_init()
        .expect("failed to initialize subscriber");
    let mozart = app();
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to localhost:8080");
    serve(listener, mozart)
        .await
        .expect("failed to start mozart");
}

async fn status() -> StatusCode {
    StatusCode::OK
}

async fn submit(Json(submission): Json<Submission>) -> SubmissionResult {
    let uuid = Uuid::new_v4();
    let span = span!(Level::DEBUG, "", %uuid);
    let _enter = span.enter();

    let temp_dir = PathBuf::from(format!("{}/{}", PARENT_DIR, uuid));

    if fs::create_dir(temp_dir.as_path()).is_err() {
        error!("could not create temporary working directory");
        return SubmissionResult::Error(SubmissionError::Internal.to_string());
    }

    let runner = TestRunner::new(temp_dir.clone());

    let response = match runner.check(submission).await {
        Ok(test_case_results) => {
            if test_case_results
                .iter()
                .all(|tc| tc.test_result == TestResult::Pass)
            {
                SubmissionResult::Pass
            } else {
                SubmissionResult::Failure(test_case_results)
            }
        }
        Err(err) => SubmissionResult::from(err),
    };

    if fs::remove_dir_all(temp_dir.as_path()).is_err() {
        error!("could not delete temporary working directory");
        return SubmissionResult::Error(SubmissionError::Internal.to_string());
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

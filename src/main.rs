use axum::{
    http::StatusCode,
    routing::{get, post},
    serve, Json, Router,
};
use error::CheckError;
use model::{Submission, TestResult};
use response::SubmitResponse;
use runner::TestRunner;
use std::{fs, path::PathBuf};
use tokio::net::TcpListener;
use uuid::Uuid;

mod error;
mod model;
mod response;
mod runner;

/// The parent directory of all test runner jobs.
const PARENT_DIR: &str = "/tmp";

fn app() -> Router {
    Router::new()
        .route("/submit", post(submit))
        .route("/status", get(status))
}

#[tokio::main]
async fn main() {
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

async fn submit(Json(submission): Json<Submission>) -> SubmitResponse {
    let temp_dir = PathBuf::from(format!("{}/{}", PARENT_DIR, Uuid::new_v4()));

    if fs::create_dir(temp_dir.as_path()).is_err() {
        return SubmitResponse::Internal;
    }

    let runner = TestRunner::new(temp_dir.clone());

    let response = match runner.check(submission) {
        Ok(test_case_results) => {
            if test_case_results
                .iter()
                .all(|tc| tc.test_result == TestResult::Pass)
            {
                SubmitResponse::Success
            } else {
                SubmitResponse::Failure(test_case_results)
            }
        }
        Err(err) => match err {
            CheckError::IOInteraction => SubmitResponse::Internal,
            CheckError::Compilation(reason) => SubmitResponse::CompilationError(reason),
        },
    };

    if fs::remove_dir_all(temp_dir.as_path()).is_err() {
        return SubmitResponse::Internal;
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

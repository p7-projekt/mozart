use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    serve, Json, Router,
};
use error::SubmissionError;
use model::Submission;
use response::SubmissionResult;
use runner::TestRunner;
use std::{fs, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, info_span};
use uuid::Uuid;

mod error;
pub mod log;
pub mod model;
pub mod response;
mod runner;
mod timeout;

/// The parent directory of all test runner jobs.
const PARENT_DIR: &str = "/tmp";

/// Defines the routing of mozart.
///
/// Mainly exists as a standalone function due to logical reasoning,
/// and to make it easier to write test cases that 'ping' the router.
pub fn app() -> Router {
    Router::new()
        .route("/submit", post(submit))
        .route("/status", get(status))
        .layer(
            TraceLayer::new_for_http().make_span_with(|_: &Request<Body>| {
                let request_id = Uuid::new_v4();
                info_span!("", %request_id)
            }),
        )
}

/// This functions starts the mozart server and will not return for as long as the server is running.
#[tokio::main]
pub async fn mozart() {
    let mozart = app();
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to localhost:8080");
    serve(listener, mozart)
        .await
        .expect("failed to start mozart");
}

/// An endpoint that exists to quickly assert whether mozart is still healthy.
///
/// This does not have any purpose for mozart itself, instead it is used as
/// part of the k3s deployment to ensure health of the individual mozart instances.
async fn status() -> StatusCode {
    info!("performed status check");
    StatusCode::OK
}

/// The endpoint used to check a given submission against a set of test cases.
pub async fn submit(Json(submission): Json<Submission>) -> SubmissionResult {
    let uuid = Uuid::new_v4();

    debug!(?submission);

    let temp_dir = PathBuf::from(format!("{}/{}", PARENT_DIR, uuid));
    info!("unique directory: {:?}", temp_dir);

    if let Err(err) = fs::create_dir(temp_dir.as_path()) {
        error!("could not create temporary working directory: {}", err);
        return SubmissionResult::from(SubmissionError::Internal);
    }

    let runner = TestRunner::new(temp_dir.clone());

    info!("checking submission");
    let response = if let Err(err) = runner.check(submission).await {
        SubmissionResult::from(err)
    } else {
        SubmissionResult::Pass
    };

    if let Err(err) = fs::remove_dir_all(temp_dir.as_path()) {
        error!("could not delete temporary working directory: {}", err);
        return SubmissionResult::from(SubmissionError::Internal);
    }

    response
}

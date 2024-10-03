use crate::model::TestCaseResult;
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Represents a response on the `/check` endpoint.
///
/// Therefore, this also represents a response to a solution that was submitted for testing.
#[derive(Serialize)]
pub enum CheckResponse {
    Success,
    Error(String),
    Failure(FailureReason),
}

impl IntoResponse for CheckResponse {
    fn into_response(self) -> Response {
        match self {
            CheckResponse::Success => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Length", 0)
                .body(Body::empty())
                .expect(""),
            CheckResponse::Error(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Body::from(msg)).into_response()
            CheckResponse::Failure(reason) => (StatusCode::OK, Json::from(reason)).into_response(),
            }
        }
    }
}

#[derive(Serialize)]
pub enum FailureReason {
    IncorrectSolution(Box<[TestCaseResult]>),
    CompilationError(String),
}

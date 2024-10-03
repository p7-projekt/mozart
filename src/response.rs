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
    /// A submission compiled without errors and passed all test cases.
    Success,

    /// A submission was not accepted due to the underlying [`FailureReason`].
    Failure(FailureReason),

    /// An internal server error occurred during handling of the solution testing.
    Error,
}

impl IntoResponse for CheckResponse {
    fn into_response(self) -> Response {
        match self {
            CheckResponse::Success => (StatusCode::OK, Body::empty()).into_response(),
            CheckResponse::Failure(reason) => (StatusCode::OK, Json::from(reason)).into_response(),
            CheckResponse::Error => {
                (StatusCode::INTERNAL_SERVER_ERROR, Body::empty()).into_response()
            }
        }
    }
}

/// The reason why a submission was not accepted.
#[derive(Serialize)]
pub enum FailureReason {
    /// The submission produced an incorrect output in one or more test cases.
    IncorrectSolution(Box<[TestCaseResult]>),

    /// The submission could not be compiled.
    CompilationError(String),
}

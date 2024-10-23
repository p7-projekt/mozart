//! Contains objects in relation to how responses are produced based on how the submission check went.

use crate::{error::SubmissionError, model::TestCaseResult};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// A submission result indicates the result of checking a given submission.
///
/// This is an outward facing object, as it is serialized to JSON in the HTTP response for a given request.
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "result", content = "reason")]
pub enum SubmissionResult {
    /// A submission successfully passed all test cases.
    #[serde(rename = "pass")]
    Pass,

    /// A submission did not pass all test cases.
    ///
    /// The `Box<[TestCaseResult]>` should contain a slice of test case results,
    /// both for passed and failed test cases. This way the frontend can
    /// correctly identify which test cases failed, and why they failed.
    #[serde(rename = "failure")]
    Failure(Box<[TestCaseResult]>),

    /// An error occured at some point during the check of the submission.
    ///
    /// This error is user facing, in that it represents errors that the user
    /// is responsible for, such at compilation errors, timeouts and the like.
    ///
    /// The `String` is the underlying [`SubmissionError`] in string format.
    #[serde(rename = "error")]
    Error(String),

    /// An internal error represents something that the user is not at fault for,
    /// for example, not being able to spawn a compilation process, or creating a file.
    InternalError,
}

impl IntoResponse for SubmissionResult {
    fn into_response(self) -> Response {
        if let SubmissionResult::InternalError = self {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        } else {
            (StatusCode::OK, Json(self)).into_response()
        }
    }
}

impl From<SubmissionError> for SubmissionResult {
    fn from(err: SubmissionError) -> Self {
        match err {
            SubmissionError::Internal => SubmissionResult::InternalError,
            SubmissionError::Failure(tcr) => SubmissionResult::Failure(tcr),
            other => SubmissionResult::Error(other.to_string()),
        }
    }
}

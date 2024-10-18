use crate::{error::SubmissionError, model::TestCaseResult};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "result", content = "reason")]
pub enum SubmissionResult {
    #[serde(rename = "pass")]
    Pass,

    #[serde(rename = "failure")]
    Failure(Box<[TestCaseResult]>),

    #[serde(rename = "error")]
    Error(String),

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
        if let SubmissionError::Internal = err {
            SubmissionResult::InternalError
        } else {
            SubmissionResult::Error(err.to_string())
        }
    }
}

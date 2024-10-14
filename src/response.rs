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
}

impl IntoResponse for SubmissionResult {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl From<SubmissionError> for SubmissionResult {
    fn from(err: SubmissionError) -> Self {
        SubmissionResult::Error(err.to_string())
    }
}

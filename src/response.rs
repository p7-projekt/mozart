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
    Failure(Box<[TestCaseResult]>),
    Error(String),
}

impl IntoResponse for CheckResponse {
    fn into_response(self) -> Response {
        match self {
            CheckResponse::Success => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Length", 0)
                .body(Body::empty())
                .expect(""),
            CheckResponse::Failure(test_cases) => {
                (StatusCode::OK, Json::from(test_cases)).into_response()
            }
            CheckResponse::Error(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Body::from(msg)).into_response()
            }
        }
    }
}

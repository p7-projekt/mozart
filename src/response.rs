use crate::model::TestCaseResult;
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

pub enum SubmitResponse {
    Success,
    Failure(Box<[TestCaseResult]>),
    CompilationError(String),
    Internal,
}

impl IntoResponse for SubmitResponse {
    fn into_response(self) -> Response {
        match self {
            SubmitResponse::Success => StatusCode::OK.into_response(),
            SubmitResponse::Failure(test_case_results) => {
                (StatusCode::OK, Json(test_case_results)).into_response()
            }
            SubmitResponse::CompilationError(reason) => {
                (StatusCode::BAD_REQUEST, Body::from(reason)).into_response()
            }
            SubmitResponse::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

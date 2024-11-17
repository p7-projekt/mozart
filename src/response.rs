//! Contains objects in relation to how responses are produced based on how the submission check went.

use crate::{error::SubmissionError, model::TestCaseResult};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{
    de::{Error, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize,
};
use std::fmt::Formatter;

/// A submission result indicates the result of checking a given submission.
///
/// This is an outward facing object, as it is serialized to JSON in the HTTP response for a given request.
#[derive(Debug, PartialEq)]
pub enum SubmissionResult {
    /// A submission successfully passed all test cases.
    Pass,

    /// A submission did not pass all test cases.
    ///
    /// The `Box<[TestCaseResult]>` should contain a slice of test case results,
    /// both for passed and failed test cases. This way the frontend can
    /// correctly identify which test cases failed, and why they failed.
    Failure(Box<[TestCaseResult]>),

    /// An error occured at some point during the check of the submission.
    ///
    /// This error is user facing, in that it represents errors that the user
    /// is responsible for, such at compilation errors, timeouts and the like.
    ///
    /// The `String` is the underlying [`SubmissionError`] in string format.
    Error(String),

    /// An internal error represents something that the user is not at fault for,
    /// for example, not being able to spawn a compilation process, or creating a file.
    InternalError,
}

impl Serialize for SubmissionResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut json = serializer.serialize_struct("SubmissionResult", 2)?;
        match self {
            SubmissionResult::Pass => {
                json.serialize_field("result", "pass")?;
            }
            SubmissionResult::Failure(test_cases) => {
                json.serialize_field("result", "failure")?;
                json.serialize_field("testCaseResults", test_cases)?;
            }
            SubmissionResult::Error(error) => {
                json.serialize_field("result", "error")?;
                json.serialize_field("message", error)?;
            }
            SubmissionResult::InternalError => {
                unreachable!("cannot happen because internal server error is not parsed to json")
            }
        }
        json.end()
    }
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

impl<'de> Deserialize<'de> for SubmissionResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SubmissionResultVisitor;

        impl<'de> Visitor<'de> for SubmissionResultVisitor {
            type Value = SubmissionResult;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                f.write_str("a valid SubmissionResult")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                match map.next_entry::<&str, &str>()? {
                    Some(("result", "pass")) => Ok(SubmissionResult::Pass),
                    Some(("result", "failure")) => {
                        if map
                            .next_key()
                            .is_ok_and(|o| o.is_some_and(|k: &str| k == "testCaseResults"))
                        {
                            let test_case_results = map.next_value()?;
                            Ok(SubmissionResult::Failure(test_case_results))
                        } else {
                            Err(Error::missing_field("testCaseResults"))
                        }
                    }
                    Some(("result", "error")) => {
                        if map
                            .next_key()
                            .is_ok_and(|o| o.is_some_and(|k: &str| k == "message"))
                        {
                            let message = map.next_value()?;
                            Ok(SubmissionResult::Error(message))
                        } else {
                            Err(Error::missing_field("message"))
                        }
                    }
                    _ => Err(Error::custom("mission result field or invalid value")),
                }
            }
        }

        deserializer.deserialize_map(SubmissionResultVisitor)
    }
}

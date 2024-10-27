//! Contains objects in relation to how responses are produced based on how the submission check went.

use crate::{error::SubmissionError, model::TestCaseResult};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{ser::SerializeStruct, Serialize};

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

#[cfg(test)]
use serde::de;

#[cfg(test)]
impl<'de> de::Deserialize<'de> for SubmissionResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct SubmissionResultVisitor;

        impl<'de> de::Visitor<'de> for SubmissionResultVisitor {
            type Value = SubmissionResult;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a valid SubmissionResult")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut result: Option<String> = None;
                let mut test_case_results: Option<Box<[TestCaseResult]>> = None;
                let mut message: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "result" => {
                            result = Some(map.next_value()?);
                        }
                        "testCaseResults" => {
                            test_case_results = Some(map.next_value()?);
                        }
                        "message" => {
                            message = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                match result.as_deref() {
                    Some("pass") => Ok(SubmissionResult::Pass),
                    Some("failure") => Ok(SubmissionResult::Failure(
                        test_case_results
                            .ok_or_else(|| de::Error::missing_field("testCaseResults"))?,
                    )),
                    Some("error") => Ok(SubmissionResult::Error(
                        message.ok_or_else(|| de::Error::missing_field("message"))?,
                    )),
                    _ => Err(de::Error::custom("unknown variant for SubmissionResult")),
                }
            }
        }

        deserializer.deserialize_map(SubmissionResultVisitor)
    }
}

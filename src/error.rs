//! Contains errors and related values.

use crate::model::TestCaseResult;
use std::time::Duration;
use thiserror::Error;

/// Used when trying to convert a path with a uuid to a string representation.
///
/// This should be used in the `expect` message of the conversion assertion.
pub const UUID_SHOULD_BE_VALID_STR: &str = "a uuid should always be valid utf8 encoding";

/// An error that occurs in relation to checking a submission.
#[derive(Debug, Error, PartialEq)]
pub enum SubmissionError {
    /// Some sort of internal error occured.
    ///
    /// It is often in relation to creating, reading from or writing to files and directories.
    ///
    /// It essentially represents any scenario where a user was not at fault for the error and would not benefit from an error message.
    #[error("an internal server error occurred")]
    Internal,

    /// There was an error during the compilation of the submitted solution.
    ///
    /// The provided `String` should contain the underlying compilation error.
    #[error("an error occurred during compilation: {0}")]
    Compilation(String),

    /// The compilation process exceeded the set timeout, and was therefore stopped prematurely.
    ///
    /// The provided `Duration` should contain the timeout duration that was exceeded.
    #[error("compilation exceeded the timeout limit of {0:?}")]
    CompileTimeout(Duration),

    /// The execution process exceeded the set timeout, and was therefore stopped prematurely.
    ///
    /// The provided `Duration` should contain the timeout duration that was exceeded.
    #[error("execution exceeded the timeout limit of {0:?}")]
    ExecuteTimeout(Duration),

    /// The submission did not pass all test cases.
    ///
    /// The underlying cause for the failure is contained within the `Box<[TestCaseResult]>`.
    ///
    /// This error variant should NOT be stringified, instead it should be converted to a `[SubmissionResult::Failure]`.
    #[error("the submission did not pass all test cases")]
    Failure(Box<[TestCaseResult]>),

    /// The execution process stopped due to an error.
    ///
    /// This could be things like syntax errors in interpretted languages.
    #[error("an error occured during execution: {0}")]
    Execution(String),
}

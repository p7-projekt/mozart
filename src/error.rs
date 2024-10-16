use std::time::Duration;

use thiserror::Error;

pub const UUID_SHOULD_BE_VALID_STR: &str = "a uuid should always be valid utf8 encoding";

/// An error that occurs in relation to checking a solution.
#[derive(Debug, Error)]
pub enum SubmissionError {
    /// This error is often in relation to creating, reading from or writing to files and directories.
    ///
    /// It essentially represents any scenario where a user was not at fault for the error and would not benefit from an error message.
    #[error("an internal server error occurred")]
    Internal,

    #[error("an error occured during compilation: {0}")]
    Compilation(String),

    #[error("compilation exceeded the timeout limit of {0:?}")]
    CompileTimeout(Duration),

    #[error("execution exceeded the timeout limit of {0:?}")]
    ExecuteTimeout(Duration),
}

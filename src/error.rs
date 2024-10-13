use thiserror::Error;

pub const UUID_SHOULD_BE_VALID_STR: &str = "a uuid should always be valid utf8 encoding";

/// An error that occurs in relation to checking a solution.
#[derive(Debug, Error)]
pub enum SubmissionError {
    /// This error is often in relation to creating, reading from or writing to files and directories.
    #[error("an error occured during an IO interaction")]
    IOInteraction,

    #[error("an error occured during compilation: {0}")]
    Compilation(String),

    #[error("a timeout happened during compilation")]
    CompileTimeout,

    #[error("a timeout happened during execution")]
    ExecuteTimeout,
}

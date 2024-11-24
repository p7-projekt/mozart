//! Contains the language specific implementation for the Python programming language.

use super::LanguageHandler;
use crate::{
    error::{SubmissionError, UUID_SHOULD_BE_VALID_STR},
    model::{Parameter, ParameterType, TestCase},
    runner::TIMEOUT,
    timeout::timeout_process,
    RESTRICTED_USER_ID,
};
use std::{path::PathBuf, process::Stdio};
use tokio::process::Command;
use tracing::{error, info};

/// The base test code for Haskell.
const PYTHON_BASE_TEST_CODE: &str = r###"
from solution import solution
from test_runner import test_checker

def main():
TEST_CASES

if __name__ == "__main__":
    exit(main())
"###;

/// The test runner for the Python implementation.
const PYTHON_TEST_RUNNER: &str = r###"
def test_checker(actual, expected):
    if actual == expected:
        print("p")
    else:
        print("f" + "," + repr(actual) + "," + repr(expected))
"###;

/// The exception handling code snippet for Python.
///
/// The `TEST_CASE` is being replace with a call to the actual test case.
/// This is done for all test cases.
const PYTHON_EXCEPTION_SNIPPET: &str = r###"
    try:
        TEST_CASE
    except Exception as e:
        print("r," + str(e))
"###;

/// The language handler for Python.
pub struct Python {
    /// A path buffer to the current working directory of a given request.
    temp_dir: PathBuf,
}

impl LanguageHandler for Python {
    fn new(temp_dir: PathBuf) -> Self {
        Self { temp_dir }
    }

    fn test_file_path(&self) -> PathBuf {
        let mut path = self.temp_dir.clone();
        path.push("main.py");

        path
    }

    fn base_test_code(&self) -> &str {
        PYTHON_BASE_TEST_CODE
    }

    fn solution_file_path(&self) -> PathBuf {
        let mut path = self.temp_dir.clone();
        path.push("solution.py");

        path
    }

    fn test_runner_file_path(&self) -> PathBuf {
        let mut path = self.temp_dir.clone();
        path.push("test_runner.py");

        path
    }

    fn test_runner_code(&self) -> &str {
        PYTHON_TEST_RUNNER
    }

    fn generate_test_cases(&self, test_cases: &[TestCase]) -> String {
        let mut generated_test_cases = Vec::with_capacity(test_cases.len());

        for test_case in test_cases {
            let formatted_input_parameters = test_case
                .input_parameters
                .iter()
                .map(|ip| self.format_parameter(ip))
                .collect::<Vec<String>>()
                .join(",");

            let formatted_output_parameters = test_case
                .output_parameters
                .iter()
                .map(|op| self.format_parameter(op))
                .collect::<Vec<String>>()
                .join(",");

            // You could easily combine this into a single format! call, I am splitting it for readability.
            let test_case = format!("        test_checker(solution({formatted_input_parameters}), ({formatted_output_parameters}))\n");
            let generated_test_case = PYTHON_EXCEPTION_SNIPPET.replace("TEST_CASE", &test_case);
            generated_test_cases.push(generated_test_case);
        }

        generated_test_cases.join("\n")
    }

    fn format_parameter(&self, parameter: &Parameter) -> String {
        match parameter.value_type {
            ParameterType::Int | ParameterType::Float => parameter.value.clone(),
            ParameterType::Char | ParameterType::String => format!(r#""{}""#, parameter.value),
            ParameterType::Bool => {
                let mut chars = parameter.value.chars();
                match chars.next() {
                    None => unreachable!("there should always be at lesat a character"),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        }
    }

    async fn run(&self) -> Result<String, SubmissionError> {
        let test_file_path = self.test_file_path();
        let test_file_str = test_file_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);

        info!("spawning execution process");
        let execution_process = Command::new("python")
            .arg(test_file_str)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .uid(*RESTRICTED_USER_ID)
            .spawn();
        let execution_handle = match execution_process {
            Ok(eh) => eh,
            Err(err) => {
                error!("could not spawn execution process: {}", err);
                return Err(SubmissionError::Internal);
            }
        };

        info!("starting execution process timeout");
        match timeout_process(TIMEOUT, execution_handle).await? {
            Some((es, output)) => {
                info!(?es);
                info!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                info!("stderr: {}", String::from_utf8_lossy(&output.stderr));

                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            None => {
                error!(
                    "execution process exceeded allowed time limit of {:?}",
                    TIMEOUT
                );
                Err(SubmissionError::ExecuteTimeout(TIMEOUT))
            }
        }
    }
}

#[cfg(test)]
mod format_parameter {
    use super::Python;
    use crate::{
        model::{Parameter, ParameterType},
        runner::LanguageHandler,
    };
    use std::path::PathBuf;

    #[test]
    fn bool_false() {
        let haskell = Python::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Bool,
            value: String::from("false"),
        };
        let expected = String::from("False");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn bool_true() {
        let haskell = Python::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Bool,
            value: String::from("true"),
        };
        let expected = String::from("True");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn int_positive() {
        let haskell = Python::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Int,
            value: String::from("100"),
        };
        let expected = String::from("100");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn int_negative() {
        let haskell = Python::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Int,
            value: String::from("-100"),
        };
        let expected = String::from("-100");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn float_positive() {
        let haskell = Python::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Float,
            value: String::from("10.0"),
        };
        let expected = String::from("10.0");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn float_negative() {
        let haskell = Python::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Float,
            value: String::from("-10.0"),
        };
        let expected = String::from("-10.0");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn char() {
        let haskell = Python::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Char,
            value: String::from("a"),
        };
        let expected = String::from("\"a\"");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn string() {
        let haskell = Python::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::String,
            value: String::from("hello"),
        };
        let expected = String::from(r#""hello""#);

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }
}

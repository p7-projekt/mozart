//! Contains the language specific implementation for the Haskell programming language.

use super::LanguageHandler;
use crate::{
    error::{SubmissionError, UUID_SHOULD_BE_VALID_STR},
    model::{Parameter, ParameterType, TestCase},
    timeout::timeout_process,
};
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::process::Command;
use tracing::{debug, error, info};

/// The timeout duration for the compilation and execution process.
const TIMEOUT: Duration = Duration::from_secs(5);

/// The base test 'runner' code for Haskell.
///
/// The markers `SOLUTION`, `TEST_CASES`, and `OUTPUT_FILE_PATH` are being substituted
/// at runtime with the request specific values.
const HASKELL_BASE_TEST_CODE: &str = r###"
SOLUTION

main = do
TEST_CASES

testChecker actual expected = do
  if actual == expected
    then appendFile "OUTPUT_FILE_PATH" ("p" ++ "\n")
    else appendFile "OUTPUT_FILE_PATH" ("f" ++ "," ++ show actual ++ "," ++ show expected ++ "\n")
"###;

/// The language handler for Haskell.
pub struct Haskell {
    /// A path buffer to the current working directory of a given request.
    temp_dir: PathBuf,
}

impl LanguageHandler for Haskell {
    fn new(temp_dir: PathBuf) -> Self {
        Self { temp_dir }
    }

    fn dir(&self) -> &PathBuf {
        &self.temp_dir
    }

    fn test_file_path(&self) -> PathBuf {
        let mut path = self.temp_dir.clone();
        path.push("Test.hs");

        path
    }

    fn base_test_code(&self) -> &str {
        HASKELL_BASE_TEST_CODE
    }

    fn generate_test_cases(&self, test_cases: &[TestCase]) -> String {
        let mut generated_test_cases = Vec::with_capacity(test_cases.len());

        for test_case in test_cases {
            let formatted_input_parameters = test_case
                .input_parameters
                .iter()
                .map(|ip| self.format_parameter(ip))
                .collect::<Vec<String>>()
                .join(" ");

            let formatted_output_parameters = test_case
                .output_parameters
                .iter()
                .map(|op| self.format_parameter(op))
                .collect::<Vec<String>>()
                .join(",");

            let generated_test_case = format!(
                "  testChecker (solution {formatted_input_parameters}) ({formatted_output_parameters})"
            );
            generated_test_cases.push(generated_test_case);
        }

        generated_test_cases.join("\n")
    }

    fn format_parameter(&self, parameter: &Parameter) -> String {
        match parameter.value_type {
            ParameterType::Int | ParameterType::Float => format!("({})", parameter.value),
            ParameterType::Char => format!("'{}'", parameter.value),
            ParameterType::String => format!(r#""{}""#, parameter.value),
            ParameterType::Bool => {
                let mut chars = parameter.value.chars();
                match chars.next() {
                    None => unreachable!(""),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        }
    }

    async fn run(&self) -> Result<(), SubmissionError> {
        let mut executable_path = self.temp_dir.clone();
        executable_path.push("test");
        let executable_str = executable_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);
        let test_file_path = self.test_file_path();
        let test_file_str = test_file_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);

        info!("spawning compilation process");
        let compile_process = Command::new("ghc")
            .args([
                "-O2",          // highest safe level of optimization (ensures same semantics)
                "-o",           // specifies the output path of the binary
                executable_str, // the output path of the binary
                test_file_str,  // the compilation target
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let compile_handle = match compile_process {
            Ok(ch) => ch,
            Err(err) => {
                error!("could not spawn compile process: {}", err);
                return Err(SubmissionError::Internal);
            }
        };

        info!("starting timeout of compilation process");
        let (compile_exit_status, compile_output) =
            match timeout_process(TIMEOUT, compile_handle).await? {
                Some((ces, co)) => (ces, co),
                None => {
                    error!(
                        "compilation process exceeded allowed time limit of {:?}",
                        TIMEOUT
                    );
                    return Err(SubmissionError::CompileTimeout(TIMEOUT));
                }
            };

        info!("checking compilation exit status");
        match compile_exit_status
            .code()
            .expect("ghc should always return exit status code")
        {
            // 0 means success
            0 => {
                info!("no compile errors");
                // if we want to return warnings from successful compilations
                // then this is the place to check stderr
            }
            // 1 means compilation error
            1 => {
                info!("compile error");
                let stderr = String::from_utf8_lossy(&compile_output.stderr);
                let mut temp_dir = self.temp_dir.clone();
                temp_dir.push("");
                let path = temp_dir.to_str().expect(UUID_SHOULD_BE_VALID_STR);
                let stripped = stderr.replace(path, "");

                debug!("compile error: {}", stripped);
                return Err(SubmissionError::Compilation(stripped));
            }
            unknown => {
                error!(
                    "compilation returned unexpected exit status '{:?}'",
                    unknown
                );
                return Err(SubmissionError::Internal);
            }
        }

        info!("spawning execution process");
        let execution_process = Command::new(executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let execution_handle = match execution_process {
            Ok(eh) => eh,
            Err(err) => {
                error!("could not spawn execution process: {}", err);
                return Err(SubmissionError::Internal);
            }
        };

        info!("starting execution process timeout");
        if timeout_process(TIMEOUT, execution_handle).await?.is_none() {
            error!(
                "execution process exceeded allowed time limit of {:?}",
                TIMEOUT
            );
            return Err(SubmissionError::ExecuteTimeout(TIMEOUT));
        }

        Ok(())
    }
}

#[cfg(test)]
mod format_parameter {
    use super::Haskell;
    use crate::{
        model::{Parameter, ParameterType},
        runner::LanguageHandler,
    };
    use std::path::PathBuf;

    #[test]
    fn bool_false() {
        let haskell = Haskell::new(PathBuf::new());
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
        let haskell = Haskell::new(PathBuf::new());
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
        let haskell = Haskell::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Int,
            value: String::from("100"),
        };
        let expected = String::from("(100)");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn int_negative() {
        let haskell = Haskell::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Int,
            value: String::from("-100"),
        };
        let expected = String::from("(-100)");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn float_positive() {
        let haskell = Haskell::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Float,
            value: String::from("10.0"),
        };
        let expected = String::from("(10.0)");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn float_negative() {
        let haskell = Haskell::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Float,
            value: String::from("-10.0"),
        };
        let expected = String::from("(-10.0)");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn char() {
        let haskell = Haskell::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::Char,
            value: String::from("a"),
        };
        let expected = String::from("'a'");

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn string() {
        let haskell = Haskell::new(PathBuf::new());
        let input = Parameter {
            value_type: ParameterType::String,
            value: String::from("hello"),
        };
        let expected = String::from(r#""hello""#);

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }
}

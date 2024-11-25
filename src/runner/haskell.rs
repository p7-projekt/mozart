//! Contains the language specific implementation for the Haskell programming language.

use super::LanguageHandler;
use crate::{
    error::{SubmissionError, UUID_SHOULD_BE_VALID_STR},
    model::{Parameter, ParameterType, TestCase},
    runner::{remove_mozart_path, TIMEOUT},
    timeout::timeout_process,
    RESTRICTED_USER_ID,
};
use std::{path::PathBuf, process::Stdio};
use tokio::process::Command;
use tracing::{debug, error, info};

/// The base test code for Haskell.
const HASKELL_BASE_TEST_CODE: &str = r###"
module Main where

import Solution
import TestRunner
import Control.Exception
import Data.List

main = do
TEST_CASES
"###;

/// The test runner for the Haskell implementation.
const HASKELL_TEST_RUNNER: &str = r###"
module TestRunner where

testChecker actual expected = do
  if actual == expected
    then putStrLn "p"
    else putStrLn ("f" ++ "," ++ show actual ++ "," ++ show expected)
"###;

/// The exception handling code snippet for Haskell.
///
/// The `TEST_CASE` is being replace with a call to the actual test case.
/// This is done for all test cases.
const HASKELL_EXCEPTION_SNIPPET: &str = r###"
  catch (TEST_CASE) (\(e :: SomeException) -> putStrLn ("r" ++ "," ++ intercalate "\\n" (lines (show e))))
"###;

/// The language handler for Haskell.
pub struct Haskell {
    /// A path buffer to the current working directory of a given request.
    temp_dir: PathBuf,
}

impl Haskell {
    async fn compile(&self, args: &[&str]) -> Result<(), SubmissionError> {
        info!("spawning compilation process");
        let compile_process = Command::new("ghc")
            .args(args)
            .arg("-O2") // best optimization level for fast vs. safe trade-off
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
                let stripped = remove_mozart_path(&stderr, self.temp_dir.clone());

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
        Ok(())
    }
}

impl LanguageHandler for Haskell {
    fn new(temp_dir: PathBuf) -> Self {
        Self { temp_dir }
    }

    fn test_file_path(&self) -> PathBuf {
        let mut path = self.temp_dir.clone();
        path.push("Main.hs");

        path
    }

    fn base_test_code(&self) -> &str {
        HASKELL_BASE_TEST_CODE
    }

    fn solution_file_path(&self) -> PathBuf {
        let mut path = self.temp_dir.clone();
        path.push("Solution.hs");

        path
    }

    fn test_runner_file_path(&self) -> PathBuf {
        let mut path = self.temp_dir.clone();
        path.push("TestRunner.hs");

        path
    }

    fn test_runner_code(&self) -> &str {
        HASKELL_TEST_RUNNER
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

            let test_case = format!(
                "testChecker (solution {formatted_input_parameters}) ({formatted_output_parameters})"
            );
            let generated_test_case = HASKELL_EXCEPTION_SNIPPET.replace("TEST_CASE", &test_case);
            generated_test_cases.push(generated_test_case);
        }

        generated_test_cases.join("\n")
    }

    fn format_parameter(&self, parameter: &Parameter) -> String {
        match parameter.value_type {
            ParameterType::Int => format!("({} :: Int)", parameter.value),
            ParameterType::Float => format!("({} :: Double)", parameter.value),
            ParameterType::Char => format!("('{}' :: Char)", parameter.value),
            ParameterType::String => format!(r#"("{}" :: String)"#, parameter.value),
            ParameterType::Bool => {
                let mut chars = parameter.value.chars();
                match chars.next() {
                    None => unreachable!("there should always be at lesat a character"),
                    Some(c) => {
                        format!(
                            "({} :: Bool)",
                            c.to_uppercase().collect::<String>() + chars.as_str()
                        )
                    }
                }
            }
        }
    }

    async fn run(&self) -> Result<String, SubmissionError> {
        info!("compiling solution");
        let solution_file_path = self.solution_file_path();
        let solution_file_str = solution_file_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);
        self.compile(&[solution_file_str]).await?;

        info!("compiling test runner");
        let test_runner_file_path = self.test_runner_file_path();
        let test_runner_file_str = test_runner_file_path
            .to_str()
            .expect(UUID_SHOULD_BE_VALID_STR);
        if self.compile(&[test_runner_file_str]).await.is_err() {
            return Err(SubmissionError::Internal);
        }

        info!("compiling test code");
        let mut executable_path = self.temp_dir.clone();
        executable_path.push("test");
        let executable_str = executable_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);
        let test_file_path = self.test_file_path();
        let test_file_str = test_file_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);
        let base_path = self
            .temp_dir
            .as_path()
            .to_str()
            .expect(UUID_SHOULD_BE_VALID_STR);

        let import_path = &format!("-i{base_path}");
        self.compile(&[
            "-o",           // flag to set the output path
            executable_str, // the path to output executable
            test_file_str,  // the absolute path of Main.hs
            import_path,    // where to look for Solution and TestRunner modules
        ])
        .await?;

        info!("spawning execution process");
        let execution_process = Command::new(executable_path)
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
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stripped = remove_mozart_path(&stdout, self.temp_dir.clone());

                Ok(stripped)
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
        let expected = String::from("(False :: Bool)");

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
        let expected = String::from("(True :: Bool)");

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
        let expected = String::from("(100 :: Int)");

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
        let expected = String::from("(-100 :: Int)");

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
        let expected = String::from("(10.0 :: Double)");

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
        let expected = String::from("(-10.0 :: Double)");

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
        let expected = String::from("('a' :: Char)");

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
        let expected = String::from(r#"("hello" :: String)"#);

        let actual = haskell.format_parameter(&input);

        assert_eq!(actual, expected);
    }
}

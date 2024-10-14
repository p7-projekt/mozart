use super::LanguageHandler;
use crate::{
    error::{SubmissionError, UUID_SHOULD_BE_VALID_STR},
    model::{Parameter, TestCase},
    timeout::timeout_process,
};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

const HASKELL_BASE_TEST_CODE: &str = r###"
SOLUTION

main = do
TEST_CASES

testChecker actual expected = do
  if actual == expected
    then appendFile "OUTPUT_FILE_PATH" ("p" ++ "\n")
    else appendFile "OUTPUT_FILE_PATH" ("f" ++ "," ++ show actual ++ "," ++ show expected ++ "\n")
"###;

pub struct Haskell {
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
                .join(" ");

            let generated_test_case = format!(
                "  testChecker (solution {formatted_input_parameters}) ({formatted_output_parameters})"
            );
            generated_test_cases.push(generated_test_case);
        }

        generated_test_cases.join("\n")
    }

    fn format_parameter(&self, parameter: &Parameter) -> String {
        match parameter.value_type.as_str() {
            "string" => format!(r#"("{}")"#, parameter.value),
            _ => format!("({})", parameter.value),
        }
    }

    async fn run(&self) -> Result<(), SubmissionError> {
        let mut executable_path = self.temp_dir.clone();
        executable_path.push("/test");
        let executable_str = executable_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);
        let test_file_path = self.test_file_path();
        let test_file_str = test_file_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);

        let Ok(compile_process) = Command::new("ghc")
            .args([
                "-O2",          // highest safe level of optimization (ensures same semantics)
                "-o",           // specifies the output path of the binary
                executable_str, // the output path of the binary
                test_file_str,  // the compilation target
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        else {
            return Err(SubmissionError::IOInteraction);
        };

        let (compile_exit_status, compile_output) =
            timeout_process(Duration::from_secs(5), compile_process)
                .await?
                .ok_or(SubmissionError::CompileTimeout)?;

        match compile_exit_status
            .code()
            .expect("ghc should always return exit status code")
        {
            // 0 means success
            0 => {
                // if we want to return warnings from successful compilations
                // then this is the place to check stderr
            }
            // 1 means compilation error
            1 => match String::from_utf8(compile_output.stderr) {
                Ok(stderr) => return Err(SubmissionError::Compilation(stderr)),
                // may never occur, and should not be this error type anyhow
                Err(_) => return Err(SubmissionError::IOInteraction),
            },
            // not correct error type
            unknown => return Err(SubmissionError::IOInteraction), // internal
        }

        let Ok(execution_handle) = Command::new(executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        else {
            return Err(SubmissionError::IOInteraction);
        };

        if timeout_process(Duration::from_secs(5), execution_handle)
            .await?
            .is_none()
        {
            return Err(SubmissionError::ExecuteTimeout);
        }

        Ok(())
    }
}

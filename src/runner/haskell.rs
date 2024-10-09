use super::LanguageHandler;
use crate::{
    error::{CheckError, UUID_SHOULD_BE_VALID_STR},
    model::{Parameter, TestCase},
};
use std::{path::PathBuf, process::Command};

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

    fn run(&self) -> Result<(), CheckError> {
        let mut executable_path = self.temp_dir.clone();
        executable_path.push("/test");
        let executable_str = executable_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);
        let test_file_path = self.test_file_path();
        let test_file_str = test_file_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);

        let Ok(compile_output) = Command::new("ghc")
            .args(["-O2", "-o", executable_str, test_file_str])
            .output()
        else {
            return Err(CheckError::IOInteraction);
        };

        let Ok(stderr) = String::from_utf8(compile_output.stderr) else {
            // this should probably not be an io interaction, and possibly may never occur
            return Err(CheckError::IOInteraction);
        };

        if stderr.contains("error:") {
            return Err(CheckError::Compilation(stderr));
        }

        if Command::new(executable_path).output().is_err() {
            return Err(CheckError::IOInteraction);
        }

        Ok(())
    }
}

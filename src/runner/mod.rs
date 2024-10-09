use crate::{
    error::{CheckError, UUID_SHOULD_BE_VALID_STR},
    model::{Parameter, Submission, TestCase, TestCaseFailureReason, TestCaseResult, TestResult},
};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

#[cfg(feature = "haskell")]
use haskell::Haskell;
#[cfg(feature = "haskell")]
mod haskell;

/// The replacement target for inserting test cases.
const TEST_CASES_TARGET: &str = "TEST_CASES";

/// The replacement target for inserting the output file path.
const OUTPUT_FILE_PATH_TARGET: &str = "OUTPUT_FILE_PATH";

/// The replacement target for inserting the submitted solution.
const SOLUTION_TARGET: &str = "SOLUTION";

pub trait LanguageHandler {
    /// Creates a new `LanguageHandler`.
    fn new(temp_dir: PathBuf) -> Self;

    /// Gets a reference to the temporary working directory of the current `LanguageHandler`.
    fn dir(&self) -> &PathBuf;

    /// Gets the path to the test file, the path should contain the file extension.
    fn test_file_path(&self) -> PathBuf;

    /// Gets the basic test runner before generated test cases, the solution, and the output file path are inserted.
    ///
    /// The test cases are inserted in place of the value in [`TEST_CASES_TARGET`].
    ///
    /// The output file path is inserted in place of the value in [`OUTPUT_FILE_PATH_TARGET`].
    ///
    /// The solution is inserted in place of the value in [`SOLUTION_TARGET`].
    fn base_test_code(&self) -> &str;

    /// Generates the language specific test cases.
    fn generate_test_cases(&self, test_cases: &[TestCase]) -> String;

    /// Formats a parameter to the necessary language specific syntax.
    fn format_parameter(&self, parameter: &Parameter) -> String;

    /// Runs the submission against the test cases.
    ///
    /// If the programming language is compiled, then this step **also** includes compilation of the source code.
    fn run(&self) -> Result<(), CheckError>;
}

pub struct TestRunner {
    #[cfg(feature = "haskell")]
    handler: Haskell,
}

impl TestRunner {
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            #[cfg(feature = "haskell")]
            handler: Haskell::new(temp_dir),
        }
    }

    pub fn check(self, submission: Submission) -> Result<Box<[TestCaseResult]>, CheckError> {
        let Ok(mut test_file) = File::create(self.handler.test_file_path()) else {
            return Err(CheckError::IOInteraction);
        };

        let mut output_file_path = self.handler.dir().clone();
        output_file_path.push("output");
        let Ok(mut output_file) = File::create_new(output_file_path.as_path()) else {
            return Err(CheckError::IOInteraction);
        };

        let output_file_path_str = output_file_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);
        let (solution, test_cases) = submission.into_inner();
        let generated_test_cases = self.handler.generate_test_cases(&test_cases);

        let final_test_code = self
            .handler
            .base_test_code()
            .replace(SOLUTION_TARGET, solution.as_str())
            .replace(TEST_CASES_TARGET, generated_test_cases.as_str())
            .replace(OUTPUT_FILE_PATH_TARGET, output_file_path_str);

        println!("{final_test_code}");

        if test_file.write_all(final_test_code.as_bytes()).is_err() {
            return Err(CheckError::IOInteraction);
        }

        self.handler.run()?;

        let mut test_output = String::new();
        if output_file.read_to_string(&mut test_output).is_err() {
            return Err(CheckError::IOInteraction);
        }

        let mut test_case_results = Vec::new();
        for (index, line) in test_output.lines().enumerate() {
            if line.trim().is_empty() {
                // not correct error
                return Err(CheckError::IOInteraction);
            }

            let test_case = &test_cases[index];

            let mut split = line.split(',');
            let result = match split.next().expect("line is not empty") {
                "p" => TestCaseResult {
                    id: test_case.id,
                    test_result: TestResult::Pass,
                },
                "f" => {
                    let (Some(actual), Some(expected)) = (split.next(), split.next()) else {
                        // not correct error type
                        return Err(CheckError::IOInteraction);
                    };

                    TestCaseResult {
                        id: test_case.id,
                        test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                            input_parameters: test_case.input_parameters.clone(),
                            actual: actual.to_string(),
                            expected: expected.to_string(),
                        }),
                    }
                }
                // not correct error type
                _ => return Err(CheckError::IOInteraction),
            };

            test_case_results.push(result);
        }

        // extrapolating that a testcase caused a runtime error
        if test_case_results.len() != test_cases.len() {
            let index = test_case_results.len();
            let test_case = &test_cases[index];
            let result = TestCaseResult {
                id: test_case.id,
                test_result: TestResult::Failure(TestCaseFailureReason::RuntimeError),
            };
            test_case_results.push(result);
        }

        // handling the remaining test cases which are considered unknown (were not run)
        for test_case in test_cases.iter().skip(test_cases.len()) {
            let result = TestCaseResult {
                id: test_case.id,
                test_result: TestResult::Unknown,
            };
            test_case_results.push(result);
        }

        Ok(test_case_results.into_boxed_slice())
    }
}

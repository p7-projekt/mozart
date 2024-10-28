//! Defines the components necessary for the language agnostic test runner to exist.

use crate::{
    error::{SubmissionError, UUID_SHOULD_BE_VALID_STR},
    model::{Parameter, Submission, TestCase, TestCaseFailureReason, TestCaseResult, TestResult},
};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use tracing::{debug, error, info};

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
    async fn run(&self) -> Result<(), SubmissionError>;
}

/// The runner responsible for testing a solution against a set of test cases.
///
/// The underlying language being tested is determined at compile time via feature flags.
pub struct TestRunner {
    #[cfg(feature = "haskell")]
    handler: Haskell,
}

impl TestRunner {
    /// Create a new test runner, based on the enabled feature flag for toggling languages.
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            #[cfg(feature = "haskell")]
            handler: Haskell::new(temp_dir),
        }
    }

    /// Checks a given submissmion against the provided test cases.
    ///
    /// # Errors
    /// An `Ok` result indicates that all test cases were passed.
    /// An `Err` result can indicate a number of things specified in the variants of `[SubmissionError]`.
    pub async fn check(self, submission: Submission) -> Result<(), SubmissionError> {
        info!("creating test file");
        let mut test_file = match File::create(self.handler.test_file_path()) {
            Ok(tf) => tf,
            Err(err) => {
                error!("could not create test file: {}", err);
                return Err(SubmissionError::Internal);
            }
        };

        info!("creating output file");
        let mut output_file_path = self.handler.dir().clone();
        output_file_path.push("output");
        let mut output_file = match File::create_new(output_file_path.as_path()) {
            Ok(of) => of,
            Err(err) => {
                error!("could not create output file: {}", err);
                return Err(SubmissionError::Internal);
            }
        };

        let output_file_path_str = output_file_path.to_str().expect(UUID_SHOULD_BE_VALID_STR);

        info!("generating language specific test cases");
        let generated_test_cases = self.handler.generate_test_cases(&submission.test_cases);
        debug!(?generated_test_cases);

        info!("combining final test code");
        let final_test_code = self
            .handler
            .base_test_code()
            .replace(SOLUTION_TARGET, &submission.solution)
            .replace(TEST_CASES_TARGET, generated_test_cases.as_str())
            .replace(OUTPUT_FILE_PATH_TARGET, output_file_path_str);
        debug!(?final_test_code);

        info!("writing test code to test file");
        if let Err(err) = test_file.write_all(final_test_code.as_bytes()) {
            error!("could not write test code to test file: {}", err);
            return Err(SubmissionError::Internal);
        }

        self.handler.run().await?;

        info!("reading output file");
        let mut test_output = String::new();
        if let Err(err) = output_file.read_to_string(&mut test_output) {
            error!("could not read test output from output file: {}", err);
            return Err(SubmissionError::Internal);
        }
        debug!(?test_output);

        let test_case_results =
            TestRunner::parse_test_output(&test_output, &submission.test_cases)?;

        if test_case_results
            .iter()
            .all(|tc| tc.test_result == TestResult::Pass)
        {
            info!("passed all test cases");
            Ok(())
        } else {
            info!("did not pass all test cases");
            Err(SubmissionError::Failure(test_case_results))
        }
    }

    /// Parses the internal format produces by running test cases against a solution.
    ///
    /// # Errors
    /// An `Ok` result indicates that the test output was correctly parsed.
    /// An `Err` result indicates that the output file was formatted in a wrong way, and was unparseable.
    fn parse_test_output(
        test_output: &str,
        test_cases: &[TestCase],
    ) -> Result<Box<[TestCaseResult]>, SubmissionError> {
        info!("parsing output file");

        // if test_output.trim().is_empty() {
        //     error!("test output file is empty");
        //     return Err(SubmissionError::Internal);
        // }

        let mut test_case_results = Vec::new();
        for (index, line) in test_output.lines().enumerate() {
            let test_case = &test_cases[index];

            if line.trim().is_empty() {
                error!("empty line in output file for test case '{}'", test_case.id);
                return Err(SubmissionError::Internal);
            }

            let mut split = line.split(',');
            let result = match split.next().expect("line should not be empty") {
                "p" => TestCaseResult {
                    id: test_case.id,
                    test_result: TestResult::Pass,
                },
                "f" => {
                    let (Some(actual), Some(expected)) = (split.next(), split.next()) else {
                        error!(
                            "test case '{}' failure did not provide actual and expected values",
                            test_case.id
                        );
                        return Err(SubmissionError::Internal);
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
                unknown => {
                    error!(
                        "unknown test outcome '{}' for test case '{}'",
                        unknown, test_case.id
                    );
                    return Err(SubmissionError::Internal);
                }
            };

            test_case_results.push(result);
        }

        // extrapolating that a testcase caused a runtime error
        if test_case_results.len() != test_cases.len() {
            let index = test_case_results.len();
            let test_case = &test_cases[index];
            info!(
                "the submission had a runtime error in test case '{:?}'",
                test_case
            );
            let result = TestCaseResult {
                id: test_case.id,
                test_result: TestResult::Failure(TestCaseFailureReason::RuntimeError),
            };
            test_case_results.push(result);
        }

        // handling the remaining test cases which are considered unknown (were not run)
        for test_case in test_cases.iter().skip(test_case_results.len()) {
            debug!("test case '{}' is unknown", test_case.id);
            let result = TestCaseResult {
                id: test_case.id,
                test_result: TestResult::Unknown,
            };
            test_case_results.push(result);
        }

        debug!(?test_case_results);
        Ok(test_case_results.into_boxed_slice())
    }
}

#[cfg(test)]
mod parse_output_file {
    use super::TestRunner;
    use crate::{
        error::SubmissionError,
        model::{
            Parameter, ParameterType, TestCase, TestCaseFailureReason, TestCaseResult, TestResult,
        },
    };

    /// A test util function to make a test case with the supplied `id` and empty parameters.
    fn empty_test_case(id: u64) -> TestCase {
        TestCase {
            id,
            input_parameters: Box::new([]),
            output_parameters: Box::new([]),
        }
    }

    #[test]
    fn empty_line() {
        let test_output = ["p", "", "p"].join("\n");
        // the parameters are not necessary for this test, only the test case id
        let test_cases = [empty_test_case(0), empty_test_case(1), empty_test_case(2)];
        let expected = Err(SubmissionError::Internal);

        let actual = TestRunner::parse_test_output(&test_output, &test_cases);

        assert_eq!(actual, expected);
    }

    #[test]
    fn failure_outcome_without_actual_and_expected() {
        let test_output = ["f"].join("\n");
        // the parameters are not necessary for this test, only the test case id
        let test_cases = [empty_test_case(0)];
        let expected = Err(SubmissionError::Internal);

        let actual = TestRunner::parse_test_output(&test_output, &test_cases);

        assert_eq!(actual, expected);
    }

    #[test]
    fn failure_outcome_with_actual_but_without_expected() {
        let test_output = ["f,5"].join("\n");
        // the parameters are not necessary for this test, only the test case id
        let test_cases = [empty_test_case(0)];
        let expected = Err(SubmissionError::Internal);

        let actual = TestRunner::parse_test_output(&test_output, &test_cases);

        assert_eq!(actual, expected);
    }

    #[test]
    fn unknown_test_output() {
        let test_output = ["p", "r"].join("\n");
        // the parameters are not necessary for this test, only the test case id
        let test_cases = [empty_test_case(0), empty_test_case(1)];
        let expected = Err(SubmissionError::Internal);

        let actual = TestRunner::parse_test_output(&test_output, &test_cases);

        assert_eq!(actual, expected);
    }

    #[test]
    fn runtime_error_in_last_test_case() -> Result<(), SubmissionError> {
        let test_output = ["p"].join("\n");
        // the parameters are not necessary for this test, only the test case id
        let test_cases = [empty_test_case(0), empty_test_case(1)];
        let expected = Box::new([
            TestCaseResult {
                id: 0,
                test_result: TestResult::Pass,
            },
            TestCaseResult {
                id: 1,
                test_result: TestResult::Failure(TestCaseFailureReason::RuntimeError),
            },
        ]);

        let actual = TestRunner::parse_test_output(&test_output, &test_cases)?;

        assert_eq!(*actual, *expected);

        Ok(())
    }

    #[test]
    fn runtime_error_in_first_test_case() -> Result<(), SubmissionError> {
        let test_output = "";
        let test_cases = [
            empty_test_case(0),
            empty_test_case(1),
            empty_test_case(2),
            empty_test_case(3),
            empty_test_case(4),
        ];
        let expected = Box::new([
            TestCaseResult {
                id: 0,
                test_result: TestResult::Failure(TestCaseFailureReason::RuntimeError),
            },
            TestCaseResult {
                id: 1,
                test_result: TestResult::Unknown,
            },
            TestCaseResult {
                id: 2,
                test_result: TestResult::Unknown,
            },
            TestCaseResult {
                id: 3,
                test_result: TestResult::Unknown,
            },
            TestCaseResult {
                id: 4,
                test_result: TestResult::Unknown,
            },
        ]);

        let actual = TestRunner::parse_test_output(test_output, &test_cases)?;

        assert_eq!(*actual, *expected);

        Ok(())
    }

    #[test]
    fn all_test_cases_passed() -> Result<(), SubmissionError> {
        let test_output = ["p", "p", "p", "p", "p"].join("\n");
        // the parameters are not necessary for this test, only the test case id
        let test_cases = [
            empty_test_case(0),
            empty_test_case(1),
            empty_test_case(2),
            empty_test_case(3),
            empty_test_case(4),
        ];
        let expected = Box::new([
            TestCaseResult {
                id: 0,
                test_result: TestResult::Pass,
            },
            TestCaseResult {
                id: 1,
                test_result: TestResult::Pass,
            },
            TestCaseResult {
                id: 2,
                test_result: TestResult::Pass,
            },
            TestCaseResult {
                id: 3,
                test_result: TestResult::Pass,
            },
            TestCaseResult {
                id: 4,
                test_result: TestResult::Pass,
            },
        ]);

        let actual = TestRunner::parse_test_output(&test_output, &test_cases)?;

        assert_eq!(*actual, *expected);

        Ok(())
    }

    #[test]
    fn all_test_cases_wrong_answer() -> Result<(), SubmissionError> {
        let test_output = ["f,5,-5", "f,10,-10", "f,7,-7", "f,-10,10", "f,-5,5"].join("\n");
        let test_cases = [
            TestCase {
                id: 0,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("5"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-5"),
                }]),
            },
            TestCase {
                id: 1,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("10"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-10"),
                }]),
            },
            TestCase {
                id: 2,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("7"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-7"),
                }]),
            },
            TestCase {
                id: 3,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-10"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("10"),
                }]),
            },
            TestCase {
                id: 4,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-5"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("5"),
                }]),
            },
        ];
        let expected = Box::new([
            TestCaseResult {
                id: 0,
                test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                    input_parameters: Box::new([Parameter {
                        value_type: ParameterType::Int,
                        value: String::from("5"),
                    }]),
                    actual: String::from("5"),
                    expected: String::from("-5"),
                }),
            },
            TestCaseResult {
                id: 1,
                test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                    input_parameters: Box::new([Parameter {
                        value_type: ParameterType::Int,
                        value: String::from("10"),
                    }]),
                    actual: String::from("10"),
                    expected: String::from("-10"),
                }),
            },
            TestCaseResult {
                id: 2,
                test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                    input_parameters: Box::new([Parameter {
                        value_type: ParameterType::Int,
                        value: String::from("7"),
                    }]),
                    actual: String::from("7"),
                    expected: String::from("-7"),
                }),
            },
            TestCaseResult {
                id: 3,
                test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                    input_parameters: Box::new([Parameter {
                        value_type: ParameterType::Int,
                        value: String::from("-10"),
                    }]),
                    actual: String::from("-10"),
                    expected: String::from("10"),
                }),
            },
            TestCaseResult {
                id: 4,
                test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                    input_parameters: Box::new([Parameter {
                        value_type: ParameterType::Int,
                        value: String::from("-5"),
                    }]),
                    actual: String::from("-5"),
                    expected: String::from("5"),
                }),
            },
        ]);

        let actual = TestRunner::parse_test_output(&test_output, &test_cases)?;

        assert_eq!(*actual, *expected);

        Ok(())
    }

    #[test]
    fn mixed_pass_and_failure_with_runtime_error() -> Result<(), SubmissionError> {
        let test_output = ["p", "f,10,-10", "p"].join("\n");
        let test_cases = [
            TestCase {
                id: 0,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("5"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-5"),
                }]),
            },
            TestCase {
                id: 1,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("10"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-10"),
                }]),
            },
            TestCase {
                id: 2,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("7"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-7"),
                }]),
            },
            TestCase {
                id: 3,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-10"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("10"),
                }]),
            },
            TestCase {
                id: 4,
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("-5"),
                }]),
                output_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("5"),
                }]),
            },
        ];
        let expected = Box::new([
            TestCaseResult {
                id: 0,
                test_result: TestResult::Pass,
            },
            TestCaseResult {
                id: 1,
                test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                    input_parameters: Box::new([Parameter {
                        value_type: ParameterType::Int,
                        value: String::from("10"),
                    }]),
                    actual: String::from("10"),
                    expected: String::from("-10"),
                }),
            },
            TestCaseResult {
                id: 2,
                test_result: TestResult::Pass,
            },
            TestCaseResult {
                id: 3,
                test_result: TestResult::Failure(TestCaseFailureReason::RuntimeError),
            },
            TestCaseResult {
                id: 4,
                test_result: TestResult::Unknown,
            },
        ]);

        let actual = TestRunner::parse_test_output(&test_output, &test_cases)?;

        assert_eq!(*actual, *expected);

        Ok(())
    }
}

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Task {
    solution: String,
    #[serde(rename = "testCases")]
    test_cases: Box<[TestCase]>,
}

impl Task {
    pub fn into_inner(self) -> (String, Box<[TestCase]>) {
        (self.solution, self.test_cases)
    }
}

#[derive(Deserialize)]
pub struct TestCase {
    id: u64,
    #[serde(rename = "inputParameters")]
    input_parameters: Box<[Parameter]>,
    #[serde(rename = "outputParameters")]
    output_parameters: Box<[Parameter]>,
}

#[derive(Deserialize, Serialize)]
pub struct Parameter {
    #[serde(rename = "valueType")]
    value_type: String,
    value: String,
}

#[derive(Serialize)]
pub struct TestCaseResult {
    id: u64,
    test_result: TestResult,
}

#[derive(Serialize)]
pub enum TestResult {
    /// The test case passed.
    Pass,

    /// The result of the test case is unknown.
    ///
    /// This is likely caused by a previous test case causing a runtime error, thereby crashing the test runner.
    Unknown,

    /// The test case did not pass.
    Failure {
        reason: TestCaseFailureReason,
        input_parameters: Box<[Parameter]>,
        actual: String,
        expected: String,
    },
}

/// The reason why a given test case failed.
#[derive(Serialize)]
pub enum TestCaseFailureReason {
    /// The answer to the test case was incorrect.
    WrongAnswer,

    /// A runtime error occured during the test case.
    RuntimeError,
}

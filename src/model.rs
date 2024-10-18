use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Submission {
    pub solution: String,
    #[serde(rename = "testCases")]
    pub test_cases: Box<[TestCase]>,
}

impl Submission {
    pub fn into_inner(self) -> (String, Box<[TestCase]>) {
        (self.solution, self.test_cases)
    }
}

#[derive(Deserialize, Debug)]
pub struct TestCase {
    pub id: u64,
    #[serde(rename = "inputParameters")]
    pub input_parameters: Box<[Parameter]>,
    #[serde(rename = "outputParameters")]
    pub output_parameters: Box<[Parameter]>,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct Parameter {
    #[serde(rename = "valueType")]
    pub value_type: String,
    pub value: String,
}

#[derive(Serialize, Debug)]
pub struct TestCaseResult {
    pub id: u64,
    #[serde(rename = "testResult", flatten)]
    pub test_result: TestResult,
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(tag = "testResult")]
pub enum TestResult {
    /// The test case passed.
    #[serde(rename = "pass")]
    Pass,

    /// The result of the test case is unknown.
    ///
    /// This is likely caused by a previous test case causing a runtime error, thereby crashing the test runner.
    #[serde(rename = "unknown")]
    Unknown,

    /// The test case did not pass.
    #[serde(rename = "failure")]
    Failure(TestCaseFailureReason),
}

/// The reason why a given test case failed.
#[derive(Serialize, PartialEq, Debug)]
#[serde(tag = "cause", content = "details")]
pub enum TestCaseFailureReason {
    /// The answer to the test case was incorrect.
    #[serde(rename = "wrongAnswer")]
    WrongAnswer {
        #[serde(rename = "inputParameters")]
        input_parameters: Box<[Parameter]>,
        actual: String,
        expected: String,
    },

    /// A runtime error occured during the test case.
    #[serde(rename = "runtimeError")]
    RuntimeError,
}

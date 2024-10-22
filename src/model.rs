use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub solution: String,
    pub test_cases: Box<[TestCase]>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    pub id: u64,
    pub input_parameters: Box<[Parameter]>,
    pub output_parameters: Box<[Parameter]>,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub value_type: ParameterType,
    pub value: String,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ParameterType {
    Bool,
    Int,
    Float,
    Char,
    String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseResult {
    pub id: u64,
    #[serde(flatten)]
    pub test_result: TestResult,
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase", tag = "testResult")]
pub enum TestResult {
    /// The test case passed.
    Pass,

    /// The result of the test case is unknown.
    ///
    /// This is likely caused by a previous test case causing a runtime error, thereby crashing the test runner.
    Unknown,

    /// The test case did not pass.
    Failure(TestCaseFailureReason),
}

/// The reason why a given test case failed.
#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase", tag = "cause", content = "details")]
pub enum TestCaseFailureReason {
    /// The answer to the test case was incorrect.
    #[serde(rename_all = "camelCase")]
    WrongAnswer {
        input_parameters: Box<[Parameter]>,
        actual: String,
        expected: String,
    },

    /// A runtime error occured during the test case.
    RuntimeError,
}

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

impl TestCase {
    pub fn to_haskell_test_case(&self) -> String {
        let f_params = self
            .input_parameters
            .iter()
            .map(|p| format!("({})", p.value))
            .collect::<Box<[String]>>()
            .join("\n");
        let expected = self
            .output_parameters
            .iter()
            .map(|p| format!("({})", p.value))
            .collect::<Box<[String]>>()
            .join("\n");

        // important space prefix due to haskell indentation requirements
        format!("  testChecker (solution {f_params}) ({expected})")
    }
}

#[derive(Deserialize, Serialize)]
pub struct Parameter {
    #[serde(rename = "valueType")]
    value_type: String,
    value: String,
}

// #[derive(Deserialize)]
// pub struct Response {
//     status: Status,
//     message: String,
//     test_case_results: Box<[TestCaseResult]>,
// }

#[derive(Serialize)]
pub struct TestCaseResult {
    id: u64,
    test_result: TestResult,
}

#[derive(Serialize)]
pub enum Response {
    Success,
    Failure(Box<[TestCaseResult]>),
    Error(String),
}

#[derive(Serialize)]
pub enum TestResult {
    Pass,
    Unknown,
    Failure {
        reason: Reason,
        input_parameters: Box<[Parameter]>,
        actual: String,
        expected: String,
    },
}

#[derive(Serialize)]
pub enum Reason {
    WrongAnswer,
    RuntimeError,
}

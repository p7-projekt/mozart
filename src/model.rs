//! All the models of mozart relate to incoming objects or outgoing objects, with a few being both.
//!
//! The models are agnostic both in terms of the underlying programming language, and the exercise being 'checked' against.

use serde::{Deserialize, Serialize};

/// A submission provided by the backend.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    /// The user submitted solution.
    pub solution: String,

    /// The test cases that must be checked for the submitted solution.
    pub test_cases: Box<[TestCase]>,
}

/// A test case for a given exercise.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    /// The test case id, this is not relevant for mozart, but knowing which test cases failed,
    /// may be important information for other aspects of the system, as such it is included.
    pub id: u64,

    /// The input arguments of a given test case, i.e. the arguments that the users solution is called with.
    ///
    /// This is a slice so as to not limit the amount of input arguments a given exercise can supply.
    pub input_parameters: Box<[Parameter]>,

    /// The output values of a given test case, i.e. the expected values to be returned in a 'correct' solution.
    ///
    /// This is a slice so as to not limit the amount of input arguments a given exercise can supply.
    pub output_parameters: Box<[Parameter]>,
}

/// A parameter.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    /// The type of the parameter.
    pub value_type: ParameterType,

    /// The value of the parameter.
    pub value: String,
}

/// The allowed types of a parameter.
///
/// During JSON deserialization of a request it is a parse error to not use one of these types as the parameter type.
///
/// For the types where it matter, they should default to the largest prelude precision of the underlying language implementation.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ParameterType {
    /// A boolean value.
    Bool,

    /// A signed 64-bit integer.
    Int,

    /// A double precision floating point value (64-bit precision).
    Float,

    /// A character, or a single character string (depending on the language).
    Char,

    /// A string or character array (depending on the language).
    String,
}

/// A test case result, indicating how a solution handled a given test case.
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseResult {
    /// The id of the test case.
    pub id: u64,

    /// The result of the test case.
    #[serde(flatten)]
    pub test_result: TestResult,
}

/// The different outcomes of a test case.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
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
#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase", tag = "cause", content = "details")]
pub enum TestCaseFailureReason {
    /// The answer to the test case was incorrect.
    #[serde(rename_all = "camelCase")]
    WrongAnswer {
        /// The input parameters of the test case, this is provided as error feedback for the frontend.
        input_parameters: Box<[Parameter]>,

        /// The value(s) produced by the submitted solution.
        actual: String,

        /// The value(s) the submitted solution should have produced.
        expected: String,
    },

    /// A runtime error occured during the test case.
    RuntimeError,
}

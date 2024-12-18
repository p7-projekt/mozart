use axum::{
    body::{to_bytes, Body},
    http::{request::Builder, Method, StatusCode},
};
use mozart::{
    app,
    model::{
        Parameter, ParameterType, Submission, TestCase, TestCaseFailureReason, TestCaseResult,
        TestResult,
    },
    response::SubmissionResult,
};
use tower::ServiceExt;

#[tokio::test]
async fn invalid_http_method() {
    let mozart = app();
    let expected_status_code = StatusCode::METHOD_NOT_ALLOWED;
    let request = Builder::new()
        .method(Method::GET)
        .uri("/submit")
        .body(Body::empty())
        .expect("failed to build request");

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to await oneshot");

    assert_eq!(actual.status(), expected_status_code);
}

#[tokio::test]
async fn no_json_header() {
    let mozart = app();
    let expected_status_code = StatusCode::UNSUPPORTED_MEDIA_TYPE;
    let request = Builder::new()
        .method(Method::POST)
        .uri("/submit")
        .body(Body::empty())
        .expect("failed to build request");

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to await oneshot");

    assert_eq!(actual.status(), expected_status_code);
}

#[tokio::test]
async fn empty_request_body() {
    let mozart = app();
    let expected_status_code = StatusCode::BAD_REQUEST;
    let request = Builder::new()
        .method(Method::POST)
        .header("Content-Type", "application/json")
        .uri("/submit")
        .body(Body::empty())
        .expect("failed to build request");

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to await oneshot");

    assert_eq!(actual.status(), expected_status_code);
}

#[tokio::test]
async fn invalid_json() {
    let mozart = app();
    let expected_status_code = StatusCode::UNPROCESSABLE_ENTITY;
    let body = serde_json::to_string(&ParameterType::Int).expect("failed to serialize body");
    let request = Builder::new()
        .method(Method::POST)
        .header("Content-Type", "application/json")
        .uri("/submit")
        .body(body)
        .expect("failed to build request");

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to await oneshot");

    assert_eq!(actual.status(), expected_status_code);
}

#[tokio::test]
async fn solution_with_all_data_types_as_input() {
    let mozart = app();
    let solution = [
            "module Solution where",
            "",
            "solution :: Int -> Double -> Bool -> Char -> String -> String",
            "solution int float bool char string = show int ++ show float ++ show bool ++ [char] ++ string"
        ].join("\n");
    let test_cases = Box::new([TestCase {
        id: 0,
        input_parameters: Box::new([
            Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            },
            Parameter {
                value_type: ParameterType::Float,
                value: String::from("5.5"),
            },
            Parameter {
                value_type: ParameterType::Bool,
                value: String::from("true"),
            },
            Parameter {
                value_type: ParameterType::Char,
                value: String::from("f"),
            },
            Parameter {
                value_type: ParameterType::String,
                value: String::from("hello"),
            },
        ]),
        output_parameters: Box::new([Parameter {
            value_type: ParameterType::String,
            value: String::from("105.5Truefhello"),
        }]),
    }]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_body = SubmissionResult::Pass;
    let expected_status = StatusCode::OK;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn solution_with_all_data_types_as_output_and_no_input() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: (Int, Double, Bool, Char, String)",
        r#"solution = (7, 8.6, True, 'a', "hhh")"#,
    ]
    .join("\n");
    let test_cases = Box::new([TestCase {
        id: 0,
        input_parameters: Box::new([]),
        output_parameters: Box::new([
            Parameter {
                value_type: ParameterType::Int,
                value: String::from("7"),
            },
            Parameter {
                value_type: ParameterType::Float,
                value: String::from("8.6"),
            },
            Parameter {
                value_type: ParameterType::Bool,
                value: String::from("true"),
            },
            Parameter {
                value_type: ParameterType::Char,
                value: String::from("a"),
            },
            Parameter {
                value_type: ParameterType::String,
                value: String::from("hhh"),
            },
        ]),
    }]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_body = SubmissionResult::Pass;
    let expected_status = StatusCode::OK;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn compilation_error() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Int -> Int",
        "solution x =",
        "  if x < 0",
        "    then x * (-1)",
        // "    else x",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("-10"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);

    if let SubmissionResult::Error(err) = actual_body {
        assert!(err.starts_with("an error occurred during compilation:"));
    } else {
        panic!("response body was not of error variant");
    }
}

#[tokio::test]
async fn compile_timeout() {
    let mozart = app();
    let repeated = "  + x\n".repeat(100000);
    let solution = [
        "module Solution where",
        "",
        "solution :: Int -> Int",
        "solution x =",
        "  x",
        repeated.as_str(),
    ]
    .join("\n");
    // the contents of the test cases are entirely irrelevant
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("-10"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);

    if let SubmissionResult::Error(err) = actual_body {
        assert!(err.starts_with("compilation exceeded the timeout limit of"));
    } else {
        panic!("response body was not of error variant");
    }
}

#[tokio::test]
async fn execution_timeout() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Int -> Int",
        "solution x = solution x",
    ]
    .join("\n");
    // the contents of the test cases are entirely irrelevant
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("-10"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);

    if let SubmissionResult::Error(err) = actual_body {
        assert!(err.starts_with("execution exceeded the timeout limit of"));
    } else {
        panic!("response body was not of error variant");
    }
}

#[tokio::test]
async fn all_test_cases_pass_int() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Int -> Int",
        "solution x = x + x",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("20"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Pass;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_pass_bool() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Bool -> Bool",
        "solution b = not b",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Bool,
                value: String::from("true"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Bool,
                value: String::from("false"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Bool,
                value: String::from("false"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Bool,
                value: String::from("true"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Pass;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_pass_float() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Double -> Double",
        "solution f = f + f",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Float,
                value: String::from("2.5"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Float,
                value: String::from("5.0"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Float,
                value: String::from("3.3"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Float,
                value: String::from("6.6"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Pass;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_pass_char() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Char -> Char",
        "solution c = c",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Char,
                value: String::from("a"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Char,
                value: String::from("a"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Char,
                value: String::from("b"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Char,
                value: String::from("b"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Pass;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_pass_string() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: String -> String",
        "solution s = s ++ s",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::String,
                value: String::from("hello"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::String,
                value: String::from("hellohello"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::String,
                value: String::from("world"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::String,
                value: String::from("worldworld"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Pass;

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_fail_int() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Int -> Int",
        "solution x = x",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("20"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("10"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Failure(Box::new([
        TestCaseResult {
            id: 0,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("10"),
                }]),
                actual: String::from("10"),
                expected: String::from("20"),
            }),
        },
        TestCaseResult {
            id: 1,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("5"),
                }]),
                actual: String::from("5"),
                expected: String::from("10"),
            }),
        },
    ]));

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_fail_bool() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Bool -> Bool",
        "solution b = b",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Bool,
                value: String::from("true"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Bool,
                value: String::from("false"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Bool,
                value: String::from("false"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Bool,
                value: String::from("true"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Failure(Box::new([
        TestCaseResult {
            id: 0,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Bool,
                    value: String::from("true"),
                }]),
                actual: String::from("True"),
                expected: String::from("False"),
            }),
        },
        TestCaseResult {
            id: 1,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Bool,
                    value: String::from("false"),
                }]),
                actual: String::from("False"),
                expected: String::from("True"),
            }),
        },
    ]));

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_fail_float() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Double -> Double",
        "solution f = f",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Float,
                value: String::from("2.2"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Float,
                value: String::from("4.4"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Float,
                value: String::from("5.0"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Float,
                value: String::from("10.0"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Failure(Box::new([
        TestCaseResult {
            id: 0,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Float,
                    value: String::from("2.2"),
                }]),
                actual: String::from("2.2"),
                expected: String::from("4.4"),
            }),
        },
        TestCaseResult {
            id: 1,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Float,
                    value: String::from("5.0"),
                }]),
                actual: String::from("5.0"),
                expected: String::from("10.0"),
            }),
        },
    ]));

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_fail_char() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Char -> Char",
        "solution c = 'a'",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Char,
                value: String::from("b"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Char,
                value: String::from("b"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Char,
                value: String::from("c"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Char,
                value: String::from("c"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Failure(Box::new([
        TestCaseResult {
            id: 0,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Char,
                    value: String::from("b"),
                }]),
                actual: String::from("'a'"),
                expected: String::from("'b'"),
            }),
        },
        TestCaseResult {
            id: 1,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Char,
                    value: String::from("c"),
                }]),
                actual: String::from("'a'"),
                expected: String::from("'c'"),
            }),
        },
    ]));

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn all_test_cases_fail_string() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: String -> String",
        "solution s = s",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::String,
                value: String::from("hello"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::String,
                value: String::from("hellohello"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::String,
                value: String::from("world"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::String,
                value: String::from("worldworld"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Failure(Box::new([
        TestCaseResult {
            id: 0,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::String,
                    value: String::from("hello"),
                }]),
                actual: String::from(r#""hello""#),
                expected: String::from(r#""hellohello""#),
            }),
        },
        TestCaseResult {
            id: 1,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::String,
                    value: String::from("world"),
                }]),
                actual: String::from(r#""world""#),
                expected: String::from(r#""worldworld""#),
            }),
        },
    ]));

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn runtime_error_in_non_last_test_case() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Int -> Int",
        "solution i = 10 `div` i",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("2"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("0"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("0"),
            }]),
        },
        TestCase {
            id: 2,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("2"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Failure(Box::new([
        TestCaseResult {
            id: 0,
            test_result: TestResult::Pass,
        },
        TestCaseResult {
            id: 1,
            test_result: TestResult::Failure(TestCaseFailureReason::RuntimeError(String::from(
                "divide by zero",
            ))),
        },
        TestCaseResult {
            id: 2,
            test_result: TestResult::Pass,
        },
    ]));

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

#[tokio::test]
async fn mixed_pass_and_fail_with_runtime_error() {
    let mozart = app();
    let solution = [
        "module Solution where",
        "",
        "solution :: Int -> Int",
        "solution x",
        "  | x >= 0 = x",
        "  | otherwise = x `div` 0",
    ]
    .join("\n");
    let test_cases = Box::new([
        TestCase {
            id: 0,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("2"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("2"),
            }]),
        },
        TestCase {
            id: 1,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("4"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
        },
        TestCase {
            id: 2,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("3"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("3"),
            }]),
        },
        TestCase {
            id: 3,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("7"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("2"),
            }]),
        },
        TestCase {
            id: 4,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("-3"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("-3"),
            }]),
        },
        TestCase {
            id: 5,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("6"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("6"),
            }]),
        },
    ]);
    let submission = Submission {
        solution,
        test_cases,
    };
    let body = serde_json::to_string(&submission).expect("failed to serialize submission");
    let request = Builder::new()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri("/submit")
        .body(Body::from(body))
        .expect("failed to build request");
    let expected_status = StatusCode::OK;
    let expected_body = SubmissionResult::Failure(Box::new([
        TestCaseResult {
            id: 0,
            test_result: TestResult::Pass,
        },
        TestCaseResult {
            id: 1,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("4"),
                }]),
                actual: String::from("4"),
                expected: String::from("5"),
            }),
        },
        TestCaseResult {
            id: 2,
            test_result: TestResult::Pass,
        },
        TestCaseResult {
            id: 3,
            test_result: TestResult::Failure(TestCaseFailureReason::WrongAnswer {
                input_parameters: Box::new([Parameter {
                    value_type: ParameterType::Int,
                    value: String::from("7"),
                }]),
                actual: String::from("7"),
                expected: String::from("2"),
            }),
        },
        TestCaseResult {
            id: 4,
            test_result: TestResult::Failure(TestCaseFailureReason::RuntimeError(String::from(
                "divide by zero",
            ))),
        },
        TestCaseResult {
            id: 5,
            test_result: TestResult::Pass,
        },
    ]));

    let actual = mozart
        .oneshot(request)
        .await
        .expect("failed to execute oneshot request");

    let actual_status = actual.status();
    let body_bytes = to_bytes(actual.into_body(), usize::MAX)
        .await
        .expect("failed to convert body to bytes");

    let actual_body: SubmissionResult =
        serde_json::from_slice(&body_bytes).expect("failed to deserialize response body");

    assert_eq!(actual_status, expected_status);
    assert_eq!(actual_body, expected_body);
}

use axum::{routing::post, serve, Json, Router};
use model::Task;
use response::CheckResponse;
use std::{
    fs::{create_dir, read_to_string, remove_dir_all, File},
    io::Write,
    path::PathBuf,
    process::Command,
};
use tokio::net::TcpListener;
use uuid::Uuid;

mod model;
mod response;

const HASKELL: &str = r###"
main = do
  writeFile "UUID_PATH/output" ""
TEST_CASES

testChecker actual expected = do
  if actual == expected
    then appendFile "UUID_PATH/output" ("pass" ++ "\n")
    else appendFile "UUID_PATH/output" ("failure" ++ "," ++ show actual ++ "," ++ show expected ++ "\n")
"###;

fn app() -> Router {
    Router::new().route("/check", post(check))
}

#[tokio::main]
async fn main() {
    let mozart = app();
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to localhost:8080");
    serve(listener, mozart)
        .await
        .expect("failed to start mozart");
}

// 0. Decode incoming json body +
// 1. generate task uuid + create temporary unique task directory +
// 2. setup haskell testing structure
// 3. generate test cases from parsed json +
// 4. write generated test cases to file +
// 5. write submitted solution to file +
// 5. compile haskell program with submitted solutio and test cases +
// 6. execute compiled haskell program +
// 7. read test case output from output file +
// 8. clean up temporary task directory +
// 9. construct task response +
async fn check(Json(task): Json<Task>) -> CheckResponse {
    let task_id = Uuid::new_v4();
    let unique_temp_dir_path = PathBuf::from(format!("/tmp/{task_id}"));

    if create_dir(unique_temp_dir_path.as_path()).is_err() {
        return CheckResponse::Error("".to_string());
    }

    let (solution, test_cases) = task.into_inner();
    let haskell_test_cases = test_cases
        .iter()
        .map(|tc| tc.to_haskell_test_case())
        .collect::<Box<[String]>>()
        .join("\n");

    let haskell_output_file_prefix = unique_temp_dir_path.to_string_lossy().to_string();
    let mut haskell_code = HASKELL
        .replace("UUID_PATH", haskell_output_file_prefix.as_str())
        .replace("TEST_CASES", haskell_test_cases.as_str());
    haskell_code.push_str(solution.as_str());

    let test_file_path = PathBuf::from(format!("{}/Test.hs", unique_temp_dir_path.display()));
    let mut test_file = match File::create_new(test_file_path.clone()) {
        Ok(f) => f,
        Err(_) => return CheckResponse::Error("".to_string()),
    };

    if test_file.write_all(haskell_code.as_bytes()).is_err() {
        return CheckResponse::Error("".to_string());
    }

    let executable_binary_path = PathBuf::from(format!("{}/test", unique_temp_dir_path.display()));
    let executable_binary_path_string = executable_binary_path.to_string_lossy().to_string();
    let test_file_path_string = test_file_path.to_string_lossy().to_string();
    let compilation = Command::new("ghc")
        .args([
            "-O2",                                  // highest level of safe optimization
            "-o",                                   // set output executable name
            executable_binary_path_string.as_str(), // name of the output executeable
            test_file_path_string.as_str(),         // the source program
        ])
        .output();

    let compilation = match compilation {
        Ok(o) => o,
        Err(e) => return CheckResponse::Error("".to_string()),
    };

    let execution = Command::new(executable_binary_path).output();
    let execution = match execution {
        Ok(o) => o,
        Err(e) => return CheckResponse::Error("".to_string()),
    };

    let test_case_results_file_path =
        PathBuf::from(format!("{}/output", unique_temp_dir_path.display()));
    let Ok(output_file_content) = read_to_string(test_case_results_file_path) else {
        return CheckResponse::Error("".to_string());
    };

    if let Err(err) = remove_dir_all(unique_temp_dir_path.as_path()) {
        return CheckResponse::Error("".to_string());
    }

    CheckResponse::Success
}

// #[cfg(test)]
// mod requests {
//     use crate::app;
//     use axum::http::{Method, Request, StatusCode};
//     use tower::ServiceExt;

//     #[tokio::test]
//     async fn content_type_not_set_to_json() {
//         let mozart = app();
//         let request = Request::builder()
//             .method(Method::POST)
//             .uri("/check")
//             .body(String::new())
//             .expect("failed to create request");

//         let response = mozart
//             .oneshot(request)
//             .await
//             .expect("failed to oneshot request");

//         assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
//     }

//     #[tokio::test]
//     async fn content_type_is_json_but_body_is_empty() {
//         let mozart = app();
//         let request = Request::builder()
//             .method(Method::POST)
//             .uri("/check")
//             .header("Content-Type", "application/json")
//             .body(String::new())
//             .expect("failed to create request");

//         let response = mozart
//             .oneshot(request)
//             .await
//             .expect("failed to oneshot request");

//         assert_eq!(response.status(), StatusCode::BAD_REQUEST);
//     }

//     #[tokio::test]
//     async fn content_type_is_json_body_is_ill_formed() {
//         let mozart = app();
//         let request = Request::builder()
//             .method(Method::POST)
//             .uri("/check")
//             .header("Content-Type", "application/json")
//             .body(String::from("}"))
//             .expect("failed to create request");

//         let response = mozart
//             .oneshot(request)
//             .await
//             .expect("failed to oneshot request");

//         assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
//     }
// }

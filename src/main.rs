use axum::{http::StatusCode, response::IntoResponse, routing::post, serve, Json, Router};
use model::Task;
use std::{
    fs::{create_dir, read_to_string, remove_dir_all, File},
    io::Write,
    path::PathBuf,
    process::Command,
};
use tokio::net::TcpListener;
use uuid::Uuid;

mod model;

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
// 9. construct task response
async fn check(Json(task): Json<Task>) -> impl IntoResponse {
    let task_id = Uuid::new_v4();
    let path = PathBuf::from(format!("/tmp/{task_id}"));
    println!("UUID: {}", path.display());

    if create_dir(path.as_path()).is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("could not create unique directory"),
        );
    }

    let Some(str_path) = path.as_path().to_str() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("could not get path as str"),
        );
    };
    let mut haskell_code = HASKELL.replace("UUID_PATH", str_path);

    let (solution, test_cases) = task.into_inner();
    let haskell_test_cases = test_cases
        .iter()
        .map(|tc| tc.to_haskell_test_case())
        .collect::<Box<[String]>>()
        .join("\n");

    haskell_code = haskell_code.replace("TEST_CASES", haskell_test_cases.as_ref());
    haskell_code.push_str(solution.as_ref());

    println!("{haskell_code}");

    let test_file_path = PathBuf::from(format!("{}/Test.hs", path.display()));
    let mut test_file = match File::create_new(test_file_path.clone()) {
        Ok(f) => f,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
    };

    if test_file.write(haskell_code.as_bytes()).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, String::new());
    }

    let executable = PathBuf::from(format!("{}/test", path.display()));
    let compilation = Command::new("ghc")
        .args([
            "-O2",                            // highest level of safe optimization
            "-o",                             // set output executable name
            executable.to_str().unwrap(),     // name of the output executeable
            test_file_path.to_str().unwrap(), // the source program
        ])
        .output();

    let compilation = match compilation {
        Ok(o) => o,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    println!("stdout: {}", String::from_utf8(compilation.stdout).unwrap());
    println!("stderr: {}", String::from_utf8(compilation.stderr).unwrap());

    let execution = Command::new(executable).output();
    let execution = match execution {
        Ok(o) => o,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    println!("stdout: {}", String::from_utf8(execution.stdout).unwrap());
    println!("stderr: {}", String::from_utf8(execution.stderr).unwrap());

    let output_file_path = PathBuf::from(format!("{}/output", path.display()));
    let Ok(output_file_content) = read_to_string(output_file_path) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("could not read output file"),
        );
    };

    if let Err(err) = remove_dir_all(path.as_path()) {
        return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
    }

    (StatusCode::OK, output_file_content)
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

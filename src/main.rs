use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    serve, Json, Router,
};
use model::Task;
use tokio::net::TcpListener;

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
    Router::new()
        .route("/submit", post(submit))
        .route("/status", get(status))
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

async fn status() -> StatusCode {
    StatusCode::OK
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
async fn submit(Json(task): Json<Task>) -> impl IntoResponse {
    StatusCode::OK
}

#[cfg(test)]
mod endpoints {
    mod status {
        use crate::app;
        use axum::{
            body::{Body, HttpBody},
            http::{request::Builder, Method, StatusCode},
        };
        use tower::ServiceExt;

        #[tokio::test]
        async fn invalid_http_method() {
            let mozart = app();
            let expected_status_code = StatusCode::METHOD_NOT_ALLOWED;
            let request = Builder::new()
                .method(Method::POST)
                .uri("/status")
                .body(Body::empty())
                .expect("failed to build request");

            let actual = mozart
                .oneshot(request)
                .await
                .expect("failed to await oneshot");

            assert_eq!(actual.status(), expected_status_code);
        }

        #[tokio::test]
        async fn body_content_does_not_affect_request() {
            let mozart = app();
            let expected_status_code = StatusCode::OK;
            let request = Builder::new()
                .method(Method::GET)
                .uri("/status")
                .body(Body::from("Hello, world!"))
                .expect("failed to build request");

            let actual = mozart
                .oneshot(request)
                .await
                .expect("failed to await oneshot");

            assert_eq!(actual.status(), expected_status_code);
        }

        #[tokio::test]
        async fn valid() {
            let mozart = app();
            let expected_status_code = StatusCode::OK;
            let request = Builder::new()
                .method(Method::GET)
                .uri("/status")
                .body(Body::empty())
                .expect("failed to build request");

            let actual = mozart
                .oneshot(request)
                .await
                .expect("failed to await oneshot");

            assert_eq!(actual.status(), expected_status_code);
            assert!(actual.body().is_end_stream());
        }
    }
}

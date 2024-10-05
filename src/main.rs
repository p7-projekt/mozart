use axum::{http::StatusCode, response::IntoResponse, routing::post, serve, Json, Router};
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
async fn check(Json(task): Json<Task>) -> impl IntoResponse {
    StatusCode::OK
}

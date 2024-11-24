use mozart::{log, mozart, RESTRICTED_USER_ID};
use tracing::info;

/// We need to initilize the logger before multithreading happens
/// otherwise local time offset cannot be determined.
///
/// As a result we initialise the logger inside a synchronous main function,
/// before calling the async tokio main function
fn main() {
    log::init();

    // this log is both for information, but also to force the user id of the restricted user to be
    // computed before mozart starts, making a failure to do so a panic condition
    info!("restricted user id is '{}'", *RESTRICTED_USER_ID);

    mozart();
}

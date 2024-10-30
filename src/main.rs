use mozart::{log, mozart};

/// We need to initilize the logger before multithreading happens
/// otherwise local time offset cannot be determined.
///
/// As a result we initialise the logger inside a synchronous main function,
/// before calling the async tokio main function
fn main() {
    log::init();

    mozart();
}

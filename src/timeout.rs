use crate::error::SubmissionError;
use std::{
    process::{Child, ExitStatus, Output},
    time::Duration,
};
use tokio::time::{sleep, Instant};

pub async fn timeout_process(
    timeout: Duration,
    mut process: Child,
) -> Result<Option<(ExitStatus, Output)>, SubmissionError> {
    let start = Instant::now();

    while process.try_wait().is_ok_and(|es| es.is_none()) && start.elapsed() < timeout {
        sleep(Duration::from_millis(100)).await;
    }

    match process.try_wait() {
        Ok(Some(exit_status)) => {
            let output = process
                .wait_with_output()
                .expect("guarded expect due to if condition");
            Ok(Some((exit_status, output)))
        }
        Ok(None) => {
            process.kill().expect("should be able to kill child");
            Ok(None)
        }
        // this is in a scenario where the process never started
        Err(_) => Err(SubmissionError::Internal),
    }
}

use crate::error::SubmissionError;
use std::{
    process::{Child, ExitStatus, Output},
    time::Duration,
};
use tokio::time::{sleep, Instant};
use tracing::{debug, error, info};

/// Calls the supplied `process` with the provided `timeout`.
///
/// If the timeout is exceeded the process is killed as part of this function call.
///
/// No matter if the process finished on its own or was killed after the timeout an `Ok` is returned.
/// The `Option` inside the `Ok` indicates whether the process exited naturally or was killed.
/// If the process exited naturally the `Some` will contain the processes exit status.
/// If the process was killed a `None` is returned as no exit status could be determined.
///
/// # Errors
/// An error can occur while attempting to wait on process, which returns a `SubmissionError::Internal`.
pub async fn timeout_process(
    timeout: Duration,
    mut process: Child,
) -> Result<Option<(ExitStatus, Output)>, SubmissionError> {
    let start = Instant::now();

    while process.try_wait().is_ok_and(|es| es.is_none()) && start.elapsed() < timeout {
        sleep(Duration::from_millis(100)).await;
    }

    debug!("finished waiting on process after {:?}", start.elapsed());

    match process.try_wait() {
        Ok(Some(exit_status)) => {
            info!("process exited before exceeding timeout");
            debug!(?exit_status);
            let output = process
                .wait_with_output()
                .expect("guarded expect due to match statement");
            Ok(Some((exit_status, output)))
        }
        Ok(None) => {
            info!("killing process after exceeding timeout");
            process.kill().expect("should be able to kill child");
            Ok(None)
        }
        Err(err) => {
            error!("unknown error from waiting on process timeout: {}", err);
            Err(SubmissionError::Internal)
        }
    }
}

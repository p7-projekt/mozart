use std::str::FromStr;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::OffsetTime;

/// The default log level applied if nothing else is specified.
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::INFO;

/// Initialises a global logging subscriber.
///
/// The only configuration is compile time based on the environment variable
/// `MOZART_LOG` which will determine the log level enabled.
pub fn init() {
    let level = level_filter(option_env!("MOZART_LOG"));
    let time = OffsetTime::local_rfc_3339().expect("could not initialize time offset");
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_timer(time)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_target(false)
        .try_init()
        .expect("failed to initialize subscriber");
}

/// Determines the level filter based on the supplied optional string slice.
fn level_filter(env_var: Option<&str>) -> LevelFilter {
    let Some(var) = env_var else {
        return DEFAULT_LOG_LEVEL;
    };

    if let Ok(level) = LevelFilter::from_str(var) {
        level
    } else {
        DEFAULT_LOG_LEVEL
    }
}

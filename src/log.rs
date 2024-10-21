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

#[cfg(test)]
mod level_filter {
    use tracing::level_filters::LevelFilter;

    use super::{level_filter, DEFAULT_LOG_LEVEL};

    #[test]
    fn none() {
        let input = None;
        let expected = DEFAULT_LOG_LEVEL;

        let actual = level_filter(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_input() {
        let input = Some("foo");
        let expected = DEFAULT_LOG_LEVEL;

        let actual = level_filter(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn off() {
        let input = Some("off");
        let expected = LevelFilter::OFF;

        let actual = level_filter(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn trace() {
        let input = Some("trace");
        let expected = LevelFilter::TRACE;

        let actual = level_filter(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn debug() {
        let input = Some("debug");
        let expected = LevelFilter::DEBUG;

        let actual = level_filter(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn info() {
        let input = Some("info");
        let expected = LevelFilter::INFO;

        let actual = level_filter(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn warn() {
        let input = Some("warn");
        let expected = LevelFilter::WARN;

        let actual = level_filter(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn error() {
        let input = Some("error");
        let expected = LevelFilter::ERROR;

        let actual = level_filter(input);

        assert_eq!(actual, expected);
    }
}

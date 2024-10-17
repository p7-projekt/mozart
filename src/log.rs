use std::str::FromStr;
use time::{format_description::well_known::Rfc3339, UtcOffset};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::OffsetTime;

const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

pub fn init() {
    let level = level_filter(option_env!("MOZART_LOG"));
    // this offset is static and will not update at runtime
    // nor will it respect summer/winter time
    let offset = UtcOffset::from_hms(2, 0, 0).expect("failed to create offset");
    let time = OffsetTime::new(offset, Rfc3339);
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

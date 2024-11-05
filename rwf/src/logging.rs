use crate::config::get_config;
use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};

pub struct Logger;

impl Logger {
    pub fn init() {
        setup_logging()
    }
}

pub fn setup_logging() {
    fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_ansi(get_config().general.tty)
        .with_file(false)
        .with_target(false)
        .finish()
        .init();
}

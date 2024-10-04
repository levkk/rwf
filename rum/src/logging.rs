use crate::config::get_config;
use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};

pub fn setup_logging() {
    fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_ansi(get_config().tty)
        .with_file(false)
        .with_target(false)
        .finish()
        .init();
}

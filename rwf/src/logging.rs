use crate::config::get_config;
use once_cell::sync::OnceCell;
use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};

static INITIALIZED: OnceCell<()> = OnceCell::new();

pub struct Logger;

impl Logger {
    pub fn init() {
        INITIALIZED.get_or_init(|| {
            setup_logging();
            get_config().log_info();

            ()
        });
    }
}

fn setup_logging() {
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

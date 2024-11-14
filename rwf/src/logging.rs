//! Wrapper around `tracing_subscriber` for logging.
//!
//! Configures application-wide logging to go to stderr at the `INFO` level.
//! If you prefer to use your own logging subscriber, don't initialize the `Logger`.
//!
//! ### Example
//!
//! ```rust
//! use rwf::prelude::*;
//!
//! Logger::init();
//! ```
use crate::config::get_config;
use once_cell::sync::OnceCell;
use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};

static INITIALIZED: OnceCell<()> = OnceCell::new();

pub struct Logger;

impl Logger {
    /// Configure logging application-wide.
    ///
    /// Calling this multiple times is safe. Logger will be initialized only once.
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

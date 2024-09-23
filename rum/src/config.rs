use aes::Aes128;
use aes_gcm_siv::{AesGcmSiv, Key};
use once_cell::sync::OnceCell;
use std::io::IsTerminal;
use time::Duration;

use crate::controller::{AllowAll, AuthHandler, MiddlewareSet};

static CONFIG: OnceCell<Config> = OnceCell::new();

pub struct Config {
    pub aes_key: Key<AesGcmSiv<Aes128>>, // AES-128 key used for encryption.
    pub cookie_max_age: Duration,
    pub tty: bool,
    pub default_auth: AuthHandler,
    pub session_duration: Duration,
    pub default_middleware: MiddlewareSet,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aes_key: Key::<AesGcmSiv<Aes128>>::default(),
            cookie_max_age: Duration::days(30),
            tty: std::io::stderr().is_terminal(),
            default_auth: AuthHandler::new(AllowAll {}),
            session_duration: Duration::days(4),
            default_middleware: MiddlewareSet::default(),
        }
    }
}

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| Config::default())
}

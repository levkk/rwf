use aes::Aes128;
use aes_gcm_siv::{AesGcmSiv, Key};
use once_cell::sync::OnceCell;
use std::io::IsTerminal;
use time::Duration;

use crate::controller::{AllowAll, AuthHandler, MiddlewareSet};
use rand::{rngs::OsRng, RngCore};

static CONFIG: OnceCell<Config> = OnceCell::new();

pub struct Config {
    pub aes_key: Key<AesGcmSiv<Aes128>>, // AES-128 key used for encryption.
    pub cookie_max_age: Duration,
    pub tty: bool,
    pub default_auth: AuthHandler,
    pub session_duration: Duration,
    pub default_middleware: MiddlewareSet,

    /// Secret key.
    ///
    /// The first 128 bits are used as the AES encryption key.
    /// The next 128 bits are used for ID obfuscation.
    /// Remaining bits can be used for other purposes (not currently reserved).
    pub secret_key: [u8; 512 / 8],
}

impl Config {
    /// Get the ID mask.
    pub fn id_mask(&self) -> &[u8] {
        &self.secret_key[128 / 8..((128 / 8) * 2)]
    }
}

impl Default for Config {
    fn default() -> Self {
        // Generate a random AES key.
        let mut secret_key = [0u8; 512 / 8];
        OsRng.fill_bytes(&mut secret_key);

        let aes_key = Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[0..128 / 8]);

        Self {
            aes_key,
            cookie_max_age: Duration::days(30),
            tty: std::io::stderr().is_terminal(),
            default_auth: AuthHandler::new(AllowAll {}),
            session_duration: Duration::days(4),
            default_middleware: MiddlewareSet::default(),
            secret_key,
        }
    }
}

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| Config::default())
}

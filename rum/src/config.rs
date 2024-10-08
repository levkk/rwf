use aes::Aes128;
use aes_gcm_siv::{AesGcmSiv, Key};
use once_cell::sync::OnceCell;
use std::io::IsTerminal;
use std::path::Path;
use time::Duration;

use crate::controller::{AllowAll, AuthHandler, MiddlewareSet};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs::read_to_string;

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Error, Debug)]
pub enum Error {
    #[error("config: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("config file not found")]
    Io(#[from] std::io::Error),

    #[error("secret key is not valid")]
    Base64(#[from] base64::DecodeError),

    #[error("secret key is incorrect length")]
    SecretKey,

    #[error("config is already loaded")]
    ConfigLoaded,
}

/// Global configuration.
pub struct Config {
    pub aes_key: Key<AesGcmSiv<Aes128>>, // AES-128 key used for encryption.
    pub secure_id_key: Key<AesGcmSiv<Aes128>>,
    pub cookie_max_age: Duration,
    pub tty: bool,
    pub default_auth: AuthHandler,
    pub session_duration: Duration,
    pub default_middleware: MiddlewareSet,
    pub cache_templates: bool,
    pub websocket: Websocket,
    pub log_queries: bool,
    pub http: Http,
}

pub struct Websocket {
    pub ping_interval: Duration,
    pub ping_timeout: Duration,
    pub ping_disconnect_count: i64,
}

pub struct Http {
    pub header_max_size: usize,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            header_max_size: 16 * 1024, // 16KB
        }
    }
}

impl Default for Websocket {
    fn default() -> Self {
        Self {
            ping_timeout: Duration::seconds(5),
            ping_interval: Duration::seconds(60),
            ping_disconnect_count: 3,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        // Generate a random AES key.
        let mut secret_key = [0u8; 256 / 8];
        OsRng.fill_bytes(&mut secret_key);

        let aes_key = Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[0..128 / 8]);
        let secure_id_key = Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[128 / 8..]);

        Self {
            aes_key,
            secure_id_key,
            cookie_max_age: Duration::days(30),
            tty: std::io::stderr().is_terminal(),
            default_auth: AuthHandler::new(AllowAll {}),
            session_duration: Duration::days(4),
            default_middleware: MiddlewareSet::default(),
            cache_templates: false,
            websocket: Websocket::default(),
            log_queries: std::env::var("RUM_LOG_QUERIES").is_ok(),
            http: Http::default(),
        }
    }
}

impl Config {
    pub async fn load() -> Result<(), Error> {
        let mut config = Config::default();
        let config_file = ConfigFile::load("Rum.toml").await?;

        let secret_key = config_file.general.secret_key()?;

        let aes_key = Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[0..128 / 8]);
        let secure_id_key = Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[128 / 8..]);

        config.aes_key = aes_key;
        config.secure_id_key = secure_id_key;
        config.log_queries = config_file.general.log_queries;

        if let Err(_) = CONFIG.set(config) {
            return Err(Error::ConfigLoaded);
        }

        Ok(())
    }

    pub fn get() -> &'static Config {
        get_config()
    }
}

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| Config::default())
}

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    general: General,
}

impl ConfigFile {
    pub async fn load(path: impl AsRef<Path> + Copy) -> Result<ConfigFile, Error> {
        let file = read_to_string(path).await?;
        let config: Self = toml::from_str(&file)?;

        Ok(config)
    }
}

#[derive(Serialize, Deserialize)]
struct General {
    #[serde(default = "General::default_secret_key")]
    secret_key: String,
    #[serde(default = "General::default_log_queries")]
    log_queries: bool,
}

impl General {
    pub fn secret_key(&self) -> Result<Vec<u8>, Error> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD.decode(&self.secret_key)?;

        if bytes.len() == 256 / 8 {
            Ok(bytes)
        } else {
            Err(Error::SecretKey)
        }
    }

    fn default_log_queries() -> bool {
        false
    }

    fn default_secret_key() -> String {
        use base64::{engine::general_purpose, Engine as _};
        use rand::Rng;

        let bytes = rand::thread_rng().gen::<[u8; 256 / 8]>();

        general_purpose::STANDARD.encode(&bytes)
    }
}

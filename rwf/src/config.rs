use aes::Aes128;
use aes_gcm_siv::{AesGcmSiv, Key};
use once_cell::sync::OnceCell;
use std::env::var;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use time::Duration;

use crate::controller::{AllowAll, AuthHandler, MiddlewareSet};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use thiserror::Error;

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

    #[error("config not found")]
    NoConfig,
}

/// Global configuration.
pub struct Config {
    path: Option<PathBuf>,
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
    pub database: Database,
}

pub struct Websocket {
    pub ping_interval: Duration,
    pub ping_timeout: Duration,
    pub ping_disconnect_count: i64,
}

pub struct Http {
    pub header_max_size: usize,
}

#[derive(Clone)]
pub struct Database {
    pub url: Option<String>,
    pub name: String,
    pub user: String,
    pub pool_size: usize,
    pub idle_timeout: Duration,
    pub checkout_timeout: Duration,
}

impl Database {
    pub fn database_url(&self) -> String {
        if let Some(url) = &self.url {
            return url.clone();
        } else {
            format!("postgresql://{}@localhost/{}", self.user, self.name)
        }
    }

    fn from_config_file(&mut self, file: &DatabaseConfig) {
        if let Some(url) = &file.url {
            self.url = Some(url.clone());
        }

        if let Some(name) = &file.name {
            self.name = name.clone();
        }

        if let Some(user) = &file.user {
            self.user = user.clone();
        }

        self.idle_timeout = Duration::seconds(file.idle_timeout as i64);
        self.checkout_timeout = Duration::seconds(file.checkout_timeout as i64);
    }
}

impl Default for Database {
    fn default() -> Self {
        let url = match var("RUM_DATABASE_URL") {
            Ok(url) => Some(url),
            Err(_) => None,
        };

        let user = match var("RUM_DATABASE_USER") {
            Ok(user) => user,
            Err(_) => var("USER").unwrap_or("postgres".into()),
        };

        let name = match var("RUM_DATABASE") {
            Ok(database) => database,
            Err(_) => user.clone(),
        };

        Self {
            url,
            user,
            name,
            pool_size: 10,
            idle_timeout: Duration::hours(1),
            checkout_timeout: Duration::seconds(5),
        }
    }
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

        #[cfg(debug_assertions)]
        let cache_templates = false;

        #[cfg(not(debug_assertions))]
        let cache_templates = true;

        Self {
            path: None,
            aes_key,
            secure_id_key,
            cookie_max_age: Duration::days(30),
            tty: std::io::stderr().is_terminal(),
            default_auth: AuthHandler::new(AllowAll {}),
            session_duration: Duration::days(4),
            default_middleware: MiddlewareSet::default(),
            cache_templates,
            websocket: Websocket::default(),
            log_queries: var("RUM_LOG_QUERIES").is_ok(),
            http: Http::default(),
            database: Database::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Config, Error> {
        let mut config = Config::default();
        let mut config_file = None;

        for name in ["rwf.toml", "Rum.toml", "Rwf.toml"] {
            let path = PathBuf::from(name);
            if path.exists() {
                config_file = Some(ConfigFile::load("Rum.toml")?);
                break;
            }

            return Err(Error::NoConfig);
        }

        let config_file = match config_file {
            Some(config_file) => config_file,
            None => return Err(Error::NoConfig),
        };

        let secret_key = config_file.general.secret_key()?;

        let aes_key = Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[0..128 / 8]);
        let secure_id_key = Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[128 / 8..]);

        config.path = Some(PathBuf::from("Rum.toml"));
        config.aes_key = aes_key;
        config.secure_id_key = secure_id_key;
        config.log_queries = config_file.general.log_queries;
        config.cache_templates = config_file.general.cache_templates;
        config
            .database
            .from_config_file(&config_file.database.unwrap_or_default());

        Ok(config)
    }

    pub fn get() -> &'static Config {
        get_config()
    }
}

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| Config::load().unwrap_or_default())
}

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    general: General,
    database: Option<DatabaseConfig>,
}

impl ConfigFile {
    pub fn load(path: impl AsRef<Path> + Copy) -> Result<ConfigFile, Error> {
        let file = read_to_string(path)?;
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
    #[serde(default = "General::default_cache_templates")]
    cache_templates: bool,
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

    fn default_cache_templates() -> bool {
        #[cfg(debug_assertions)]
        return false;
        #[cfg(not(debug_assertions))]
        return true;
    }
}

#[derive(Serialize, Deserialize, Default)]
struct DatabaseConfig {
    url: Option<String>,
    name: Option<String>,
    user: Option<String>,
    #[serde(default = "DatabaseConfig::default_idle_timeout")]
    idle_timeout: usize,
    #[serde(default = "DatabaseConfig::default_checkout_timeout")]
    checkout_timeout: usize,
}

impl DatabaseConfig {
    fn default_idle_timeout() -> usize {
        3600
    }

    fn default_checkout_timeout() -> usize {
        5
    }
}

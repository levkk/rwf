use aes::Aes128;
use aes_gcm_siv::{AesGcmSiv, Key};
use once_cell::sync::OnceCell;
use std::env::var;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use time::Duration;

use crate::controller::middleware::csrf::Csrf;
use crate::controller::middleware::{request_tracker::RequestTracker, Middleware};
use crate::controller::{AuthHandler, MiddlewareSet};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use thiserror::Error;

static CONFIG: OnceCell<ConfigFile> = OnceCell::new();

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

pub fn get_config() -> &'static ConfigFile {
    CONFIG.get_or_init(|| ConfigFile::load_default())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    #[serde(skip)]
    pub path: Option<PathBuf>,

    #[serde(default = "General::default")]
    pub general: General,

    #[serde(default = "DatabaseConfig::default")]
    pub database: DatabaseConfig,

    #[serde(default = "WebsocketConfig::default")]
    pub websocket: WebsocketConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            path: None,
            general: General::default(),
            database: DatabaseConfig::default(),
            websocket: WebsocketConfig::default(),
        }
        .transform()
    }
}

impl ConfigFile {
    pub fn load_default() -> Self {
        for path in ["rwf.toml", "Rwf.toml", "Rum.toml"] {
            let path = Path::new(path);
            if path.is_file() {
                return Self::load(path).unwrap_or_default();
            }
        }

        Self::default()
    }

    pub fn load(path: impl AsRef<Path> + Copy) -> Result<ConfigFile, Error> {
        let file = read_to_string(path)?;
        let mut config: Self = toml::from_str(&file)?;
        config.path = Some(path.as_ref().to_owned());

        let mut config = config.transform();

        let secret_key = config.general.secret_key()?;

        config.general.aes_key =
            Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[0..128 / 8]);
        config.general.secure_id_key =
            Key::<AesGcmSiv<Aes128>>::clone_from_slice(&secret_key[128 / 8..]);

        Ok(config)
    }

    fn transform(mut self) -> Self {
        let mut default_middleware = vec![];

        // Request tracker always first. We want it to always run.
        if self.general.track_requests {
            default_middleware.push(RequestTracker::new().middleware());
        }

        if self.general.csrf_protection {
            default_middleware.push(Csrf::new().middleware());
        }

        self.general.default_middleware = MiddlewareSet::without_default(default_middleware);
        self
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct General {
    #[serde(default = "General::default_secret_key")]
    secret_key: String,
    #[serde(skip)]
    pub aes_key: Key<AesGcmSiv<Aes128>>,
    #[serde(skip)]
    pub secure_id_key: Key<AesGcmSiv<Aes128>>,
    #[serde(default = "General::default_log_queries")]
    pub log_queries: bool,
    #[serde(default = "General::default_cache_templates")]
    pub cache_templates: bool,
    #[serde(default = "General::default_track_requests")]
    pub track_requests: bool,
    #[serde(default = "General::default_csrf_protection")]
    pub csrf_protection: bool,
    #[serde(default = "General::default_cookie_max_age")]
    cookie_max_age: usize,
    #[serde(default = "General::default_session_duration")]
    session_duration: usize,
    #[serde(default = "General::default_tty")]
    pub tty: bool,
    #[serde(default = "General::default_header_max_size")]
    pub header_max_size: usize,
    #[serde(skip)]
    pub default_auth: AuthHandler,
    #[serde(skip)]
    pub default_middleware: MiddlewareSet,
}

impl Default for General {
    fn default() -> Self {
        Self {
            secret_key: General::default_secret_key(),
            aes_key: Key::<AesGcmSiv<Aes128>>::default(),
            secure_id_key: Key::<AesGcmSiv<Aes128>>::default(),
            log_queries: General::default_log_queries(),
            cache_templates: General::default_cache_templates(),
            track_requests: General::default_track_requests(),
            csrf_protection: General::default_csrf_protection(),
            cookie_max_age: General::default_cookie_max_age(),
            session_duration: General::default_session_duration(),
            tty: General::default_tty(),
            header_max_size: General::default_header_max_size(),
            default_auth: AuthHandler::default(),
            default_middleware: MiddlewareSet::without_default(vec![]),
        }
    }
}

fn true_from_env(name: &str) -> bool {
    if let Ok(var) = var(name) {
        ["1", "true"].contains(&var.as_str())
    } else {
        false
    }
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
        if true_from_env("RWF_LOG_QUERIES") {
            return true;
        }

        false
    }

    fn default_secret_key() -> String {
        if let Ok(key) = var("RWF_SECRET_KEY") {
            return key;
        }

        use base64::{engine::general_purpose, Engine as _};
        use rand::Rng;

        let bytes = rand::thread_rng().gen::<[u8; 256 / 8]>();

        general_purpose::STANDARD.encode(&bytes)
    }

    fn default_cache_templates() -> bool {
        if true_from_env("RWF_CACHE_TEMPLATES") {
            return true;
        }

        #[cfg(debug_assertions)]
        return false;
        #[cfg(not(debug_assertions))]
        return true;
    }

    fn default_track_requests() -> bool {
        if true_from_env("RWF_TRACK_REQUESTS") {
            return true;
        }
        false
    }

    fn default_csrf_protection() -> bool {
        if true_from_env("RWF_CSRF_PROTECTION") {
            return true;
        }
        true
    }

    fn default_cookie_max_age() -> usize {
        Duration::days(30).whole_milliseconds() as usize
    }

    pub fn cookie_max_age(&self) -> Duration {
        Duration::milliseconds(self.cookie_max_age as i64)
    }

    pub fn session_duration(&self) -> Duration {
        Duration::milliseconds(self.session_duration as i64)
    }

    fn default_session_duration() -> usize {
        Duration::weeks(4).whole_milliseconds() as usize
    }

    fn default_tty() -> bool {
        std::io::stderr().is_terminal()
    }

    fn default_header_max_size() -> usize {
        16 * 1024 // 16K
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WebsocketConfig {
    #[serde(default = "WebsocketConfig::default_ping_timeout")]
    pub ping_timeout: usize,
    #[serde(default = "WebsocketConfig::default_ping_interval")]
    pub ping_interval: usize,
    #[serde(default = "WebsocketConfig::default_disconnect_count")]
    pub ping_disconnect_count: usize,
}

impl Default for WebsocketConfig {
    fn default() -> Self {
        Self {
            ping_timeout: Self::default_ping_timeout(),
            ping_interval: Self::default_ping_interval(),
            ping_disconnect_count: Self::default_disconnect_count(),
        }
    }
}

impl WebsocketConfig {
    fn default_ping_timeout() -> usize {
        Duration::seconds(5).whole_milliseconds() as usize
    }

    pub fn ping_timeout(&self) -> Duration {
        Duration::milliseconds(self.ping_timeout as i64)
    }

    fn default_ping_interval() -> usize {
        Duration::seconds(60).whole_milliseconds() as usize
    }

    pub fn ping_interval(&self) -> Duration {
        Duration::milliseconds(self.ping_interval as i64)
    }

    fn default_disconnect_count() -> usize {
        3
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    url: Option<String>,
    name: Option<String>,
    user: Option<String>,
    #[serde(default = "DatabaseConfig::default_idle_timeout")]
    pub idle_timeout: usize,
    #[serde(default = "DatabaseConfig::default_checkout_timeout")]
    pub checkout_timeout: usize,
    #[serde(default = "DatabaseConfig::default_pool_size")]
    pub pool_size: usize,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            url: None,
            name: None,
            user: None,
            idle_timeout: DatabaseConfig::default_idle_timeout(),
            checkout_timeout: DatabaseConfig::default_checkout_timeout(),
            pool_size: DatabaseConfig::default_pool_size(),
        }
    }
}

impl DatabaseConfig {
    fn default_idle_timeout() -> usize {
        3600 * 1000
    }

    pub fn idle_timeout(&self) -> Duration {
        Duration::milliseconds(self.idle_timeout as i64)
    }

    fn default_checkout_timeout() -> usize {
        5 * 1000
    }

    pub fn checkout_timeout(&self) -> Duration {
        Duration::milliseconds(self.checkout_timeout as i64)
    }

    fn default_pool_size() -> usize {
        10
    }

    pub fn database_url(&self) -> String {
        match self.url {
            Some(ref url) => url.clone(),
            None => match var("RWF_DATABASE_URL") {
                Ok(url) => url,
                Err(_) => {
                    let user = self.user.clone().unwrap_or(match var("RWF_DATABASE_USER") {
                        Ok(user) => user,
                        Err(_) => match var("USER") {
                            Ok(user) => user,
                            Err(_) => "postgres".into(),
                        },
                    });
                    let name = self.name.clone().unwrap_or(match var("RWF_DATABASE") {
                        Ok(name) => name,
                        Err(_) => user.clone(),
                    });

                    format!("postgresql://{}@localhost/{}", user, name)
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs::File, io::Write};
    use tempdir::TempDir;

    #[test]
    fn test_load_config() {
        for config_path in ["Rwf.toml", "rwf.toml", "Rum.toml"] {
            let tmp_dir = TempDir::new("test").unwrap();
            let path = tmp_dir.path();

            std::env::set_current_dir(path).unwrap();

            let config = r#"
[general]
cache_templates = true

[database]
name = "test"
    "#;
            let path = path.join(config_path);
            let mut file = File::create(path).unwrap();
            file.write_all(config.as_bytes()).unwrap();

            let config = ConfigFile::load_default();
            assert_eq!(config.path, Some(PathBuf::from(config_path)));
        }
    }
}

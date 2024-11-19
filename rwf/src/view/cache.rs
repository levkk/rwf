//! Global template cache.
//!
//! Using the cache ensures that templates are only compiled once, increasing their
//! execution speed considerably.
//! The template cache is enabled by default in production (`release`), and disabled
//! in development (`debug`).
//!
//! [`Template::load`] uses the template cache automatically.
use super::{template::Error, Template};
use crate::config::get_config;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::{Mutex, MutexGuard};

static TEMPLATES: Lazy<Mutex<Templates>> = Lazy::new(|| Mutex::new(Templates::new()));

/// Templates cache.
pub struct Templates {
    templates: HashMap<PathBuf, Arc<Template>>,
}

impl Templates {
    /// Create new empty template cache.
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Retrieve a template from the cache. If the template doesn't exist, it will be fetched
    /// from disk and compiled.
    ///
    /// While this has to be done while holding the global template lock, this operation will be
    /// fast once most templates are cached.
    /// Holding the global lock while reading the template from disk
    /// prevents the thundering herd problem.
    pub fn get(&mut self, path: impl AsRef<Path> + Copy) -> Result<Arc<Template>, Error> {
        let cache_templates = get_config().general.cache_templates;

        if let Some(t) = self.templates.get(path.as_ref()) {
            return Ok(t.clone());
        }

        let template = Arc::new(Template::new(path)?);

        if cache_templates {
            self.templates.insert(path.as_ref().to_owned(), template);
            Ok(self.templates.get(path.as_ref()).unwrap().clone())
        } else {
            Ok(template)
        }
    }

    /// Obtain a lock to the global template cache.
    pub fn cache() -> MutexGuard<'static, Templates> {
        TEMPLATES.lock()
    }
}

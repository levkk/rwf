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
    /// # Implementation note
    ///
    /// While this has to be done while holding the global template lock, this operation will be
    /// fast once most templates are cached. Holding the global lock while reading the template from disk
    /// prevents the thundering herd problem in busy applications.
    ///
    /// # Preload templates
    ///
    /// To avoid cold start delays, preload your templates at runtime with [`Templates::preload`].
    pub fn get(&mut self, path: impl AsRef<Path> + Copy) -> Result<Arc<Template>, Error> {
        let cache_templates = get_config().general.cache_templates;

        if let Some(t) = self.templates.get(path.as_ref()) {
            return Ok(t.clone());
        }

        let template = Arc::new(Template::new(path)?);

        if cache_templates {
            self.templates
                .insert(path.as_ref().to_owned(), template.clone());
            Ok(template)
        } else {
            Ok(template)
        }
    }

    /// Compile the template from source and store it in the cache. Requires a globally unique
    /// path key, which doesn't have to point to anything that actually exists.
    pub fn from_str(
        &mut self,
        path: impl AsRef<Path> + Copy,
        src: &str,
    ) -> Result<Arc<Template>, Error> {
        let cache_templates = get_config().general.cache_templates;

        if let Some(t) = self.templates.get(path.as_ref()) {
            return Ok(t.clone());
        }

        let template = Arc::new(Template::from_str(src)?);

        if cache_templates {
            self.templates
                .insert(path.as_ref().to_owned(), template.clone());
            Ok(template)
        } else {
            Ok(template)
        }
    }

    /// Load and compile a template and store it in the cache. This will ensure
    /// the template is loaded from cache at runtime.
    pub fn preload(&mut self, path: impl AsRef<Path> + Copy) -> Result<(), Error> {
        let template = Arc::new(Template::new(path)?);
        self.templates
            .insert(path.as_ref().to_owned(), template.clone());

        Ok(())
    }

    /// Load and compile a template from source. A template referenced with this path at runtime
    /// will be fetched from cache.
    ///
    /// This method can be used to load templates that may not be available at runtime into memory.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::view::Templates;
    /// Templates::cache()
    ///     .preload_str("templates/index.html", "<h1><%= title %></h1>")
    ///         .unwrap();
    /// ```
    pub fn preload_str(&mut self, path: impl AsRef<Path> + Copy, src: &str) -> Result<(), Error> {
        let template = Arc::new(Template::from_str(src)?);
        self.templates
            .insert(path.as_ref().to_owned(), template.clone());

        Ok(())
    }

    /// Obtain a lock to the global template cache.
    pub fn cache() -> MutexGuard<'static, Templates> {
        TEMPLATES.lock()
    }
}

//! Wrapper around a [`Controller`], allowing it to be routed
//! to at request time.
//!
//! See [`crate::http::router`] documentation for routing implementation details.
use super::{
    path::{PathType, PathWithRegex},
    Path,
};
use crate::controller::Controller;

use std::ops::Deref;

/// Route handler.
///
/// You don't have to use methods below to create route handlers. Rwf provides handy macros
/// which make this experience more ergonomic.
pub struct Handler {
    path: PathWithRegex,
    name: Option<String>,
    controller: Box<dyn Controller>,
    rank: i64,
}

impl Handler {
    /// Create new route handler for the specified path, controller and path type.
    pub fn new(path: &str, controller: impl Controller + 'static, path_type: PathType) -> Self {
        Self {
            path: Path::parse(path).unwrap().with_regex(path_type).unwrap(),
            controller: Box::new(controller),
            name: None,
            rank: 0,
        }
    }

    /// Get the handler rank in the routing hierarchy.
    pub fn rank(&self) -> i64 {
        self.rank
    }

    /// Create a REST route handler. This creates several routes that all map to the same controller, supporting all 6 REST verbs.
    ///
    /// Use the `rest!` macro instead:
    ///
    /// ```rust,ignore
    /// rest!("/users" => UsersController)
    /// ```
    pub fn rest(path: &str, controller: impl Controller + 'static) -> Self {
        Self::new(path, controller, PathType::Rest)
    }

    /// Create a REST route handling this and all child paths.
    ///
    /// This is useful to create catch-all routes.
    pub fn wildcard(path: &str, controller: impl Controller + 'static) -> Self {
        Self::new(path, controller, PathType::Wildcard).with_rank(-20)
    }

    /// Create a regular route.
    ///
    /// Use the `route!` macro instead:
    ///
    /// ```rust,ignore
    /// route!("/users" => UsersController)
    /// ```
    pub fn route(path: &str, controller: impl Controller + 'static) -> Self {
        Self::new(path, controller, PathType::Route)
    }

    /// Set the route name.
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Get the path and its correspoding regex, used in the router.
    pub fn path_with_regex(&self) -> &PathWithRegex {
        &self.path
    }

    /// Get the handler's path.
    pub fn path(&self) -> &Path {
        self.path.deref()
    }

    /// Add a rank to the handler, overring its default
    /// hierarchy in the router.
    pub fn with_rank(mut self, rank: i64) -> Self {
        self.rank = rank;
        self
    }

    /// Get the controller name served by this route handler.
    pub fn controller_name(&self) -> &'static str {
        self.deref().controller_name()
    }
}

impl Deref for Handler {
    type Target = Box<dyn Controller>;

    fn deref(&self) -> &Self::Target {
        &self.controller
    }
}

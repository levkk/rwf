use super::{
    path::{PathType, PathWithRegex},
    Path,
};
use crate::controller::Controller;

use std::ops::Deref;

/// Route handler.
pub struct Handler {
    path: PathWithRegex,
    name: Option<String>,
    controller: Box<dyn Controller>,
    rank: i64,
}

impl Handler {
    pub fn new(path: &str, controller: impl Controller + 'static, path_type: PathType) -> Self {
        Self {
            path: Path::parse(path).unwrap().with_regex(path_type).unwrap(),
            controller: Box::new(controller),
            name: None,
            rank: 0,
        }
    }

    pub fn rank(&self) -> i64 {
        self.rank
    }

    pub fn rest(path: &str, controller: impl Controller + 'static) -> Self {
        Self::new(path, controller, PathType::Rest)
    }

    pub fn wildcard(path: &str, controller: impl Controller + 'static) -> Self {
        Self::new(path, controller, PathType::Wildcard).with_rank(-20)
    }

    pub fn route(path: &str, controller: impl Controller + 'static) -> Self {
        Self::new(path, controller, PathType::Route)
    }

    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn path_with_regex(&self) -> &PathWithRegex {
        &self.path
    }

    pub fn path(&self) -> &Path {
        self.path.deref()
    }

    pub fn with_rank(mut self, rank: i64) -> Self {
        self.rank = rank;
        self
    }

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

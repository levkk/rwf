use super::{path::PathWithRegex, Path};
use crate::controller::Controller;

use std::ops::Deref;

pub struct Handler {
    path: PathWithRegex,
    name: Option<String>,
    controller: Box<dyn Controller>,
    rank: i64,
}

impl Handler {
    pub fn new(path: &str, controller: impl Controller + 'static) -> Self {
        Self {
            path: Path::parse(path).unwrap().with_regex().unwrap(),
            controller: Box::new(controller),
            rank: -20,
            name: None,
        }
    }

    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn rank(mut self, rank: i64) -> Self {
        self.rank = rank;
        self
    }

    pub fn path_with_regex(&self) -> &PathWithRegex {
        &self.path
    }

    pub fn path(&self) -> &Path {
        self.path.deref()
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

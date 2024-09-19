use super::Path;
use crate::controller::Controller;
use std::cmp::{Ordering, PartialOrd};

use std::ops::Deref;

pub struct Handler {
    path: Path,
    name: Option<String>,
    controller: Box<dyn Controller>,
    rank: i64,
}

impl Handler {
    pub fn new(path: &str, controller: Box<dyn Controller>) -> Self {
        Self {
            path: Path::parse(path).unwrap().root(),
            controller,
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn controller_name(&self) -> &'static str {
        self.deref().controller_name()
    }
}

impl PartialEq for Handler {
    fn eq(&self, other: &Self) -> bool {
        match self.path.eq(&other.path) {
            true => self.rank == other.rank,
            false => false,
        }
    }
}

impl PartialOrd for Handler {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.path.partial_cmp(&other.path) {
            Some(Ordering::Equal) => self.rank.partial_cmp(&other.rank),
            ordering => ordering,
        }
    }
}

impl Eq for Handler {}

impl Ord for Handler {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl Deref for Handler {
    type Target = Box<dyn Controller>;

    fn deref(&self) -> &Self::Target {
        &self.controller
    }
}

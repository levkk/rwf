use super::{Path, Request, Response, ToResource};
use crate::controller::{Controller, Error};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

pub struct Handler {
    path: Path,
    controller: Box<dyn Controller>,
}

impl Handler {
    pub fn new(path: &str, controller: Box<dyn Controller>) -> Self {
        Self {
            path: Path::parse(path).unwrap(),
            controller,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
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

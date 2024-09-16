use super::Path;
use crate::controller::Controller;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

pub struct Handler<T> {
    path: Path,
    controller: Box<dyn Controller<Resource = T>>,
}

impl<T: FromStr> Handler<T> {
    pub fn new(path: &str, controller: Box<dyn Controller<Resource = T>>) -> Self {
        Self {
            path: Path::parse(path).unwrap(),
            controller,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<T: FromStr> Deref for Handler<T> {
    type Target = Box<dyn Controller<Resource = T>>;

    fn deref(&self) -> &Self::Target {
        &self.controller
    }
}

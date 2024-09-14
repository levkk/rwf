use super::Path;
use crate::controller::Controller;

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
}

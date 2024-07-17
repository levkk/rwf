
use super::{Error, Job};
use std::collections::HashMap;
use std::future::Future;

pub struct Worker {
    jobs: HashMap<String, Box<dyn Fn() -> Result<Box<dyn Future<Output = ()>>, Error>>>,
}

impl Worker {
    // pub fn add(&mut self, name: &str, f: fn() -> Result<(), Error>) {
    //     self.jobs.insert(name.to_string(), Box::new(f));
    // }

    // pub fn execute(job: &str)
}

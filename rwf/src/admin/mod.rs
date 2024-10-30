pub mod controllers;
pub mod models;

use crate::controller::{Controller, Engine};

pub fn engine() -> Engine {
    Engine::new(vec![
        controllers::Index::default().route("/"),
        controllers::Jobs::default().route("/jobs"),
    ])
}

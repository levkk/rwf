use models::{ModelController, ModelsController};
use rwf::controller::Engine;
use rwf::prelude::*;

mod controllers;
use controllers::*;

pub fn engine() -> Engine {
    Engine::new(vec![
        route!("/" => Index),
        route!("/jobs" => Jobs),
        route!("/requests" => Requests),
        route!("/models" => ModelsController),
        route!("/models/model" => ModelController),
    ])
}

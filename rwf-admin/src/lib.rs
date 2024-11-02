use rwf::controller::Engine;
use rwf::prelude::*;

mod controllers;
use controllers::*;

use controllers::models::{ModelController, ModelsController};

mod models;
mod views;

pub fn engine() -> Engine {
    Engine::new(vec![
        route!("/" => Index),
        route!("/jobs" => Jobs),
        route!("/requests" => Requests),
        route!("/models" => ModelsController),
        route!("/models/model" => ModelController),
    ])
}

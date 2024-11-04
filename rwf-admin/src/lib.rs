use rwf::controller::Engine;
use rwf::prelude::*;

mod controllers;
use controllers::*;

mod models;
// mod views;

pub fn engine() -> Engine {
    Engine::new(vec![
        route!("/" => index::Index),
        route!("/jobs" => jobs::Jobs),
        route!("/requests" => requests::Requests),
        route!("/models" => controllers::models::ModelsController),
        route!("/models/model" => controllers::models::ModelController),
        route!("/models/new" => controllers::models::NewModelController),
    ])
}

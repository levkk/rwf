use rwf::controller::Engine;
use rwf::prelude::*;

mod controllers;
use controllers::*;

pub fn engine() -> Engine {
    Engine::new(vec![
        route!("/" => Index),
        route!("/jobs" => Jobs),
        route!("/requests" => Requests),
    ])
}

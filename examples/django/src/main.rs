use rwf::controller::WsgiController;
use rwf::http::Server;
use rwf::prelude::*;

#[derive(Default)]
struct RustIndex;

#[async_trait]
impl Controller for RustIndex {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("This is served by Rust."))
    }
}

#[tokio::main]
async fn main() {
    Logger::init();

    // Set PYTHONPATH to where the Django app is.
    let cwd = std::env::current_dir().unwrap();
    let pythonpath = cwd.join("todo");
    std::env::set_var("PYTHONPATH", pythonpath.display().to_string());

    Server::new(vec![
        // Serve /rust with Rwf.
        route!("/rust" => RustIndex),
        // Serve every other path with Django.
        WsgiController::new("todo.wsgi").wildcard("/"),
    ])
    .launch("0.0.0.0:8002")
    .await
    .unwrap();
}

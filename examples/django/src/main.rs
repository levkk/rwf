use rwf::controller::WsgiController;
use rwf::http::Server;
use rwf::prelude::*;

use pyo3::prelude::*;
use std::path::Path;

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
    tracing::info!("Adding \"todo\" to PYTHONPATH");

    let cwd = std::env::current_dir().unwrap();
    let pythonpath = cwd.join("todo");
    std::env::set_var("PYTHONPATH", pythonpath.display().to_string());

    // Make virtualenv work
    for py in ["3.8", "3.9", "3.10", "3.11", "3.12", "3.13"] {
        let venv = Path::new("venv/lib/")
            .join(&format!("python{}", py))
            .join("site-packages");

        if venv.exists() {
            tracing::info!("Importing packages from \"{}\" (venv)", venv.display());

            Python::with_gil(|py| {
                let sys = py.import_bound("sys").unwrap();
                let path = sys.getattr("path").unwrap();
                path.call_method1("append", (venv.display().to_string(),))
                    .unwrap();
            });
            break;
        }
    }

    Server::new(vec![
        // Serve /rust with Rwf.
        route!("/rust" => RustIndex),
        // Serve every other path with Django.
        WsgiController::new("todo.wsgi").wildcard("/"),
    ])
    .launch()
    .await
    .unwrap();
}

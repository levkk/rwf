use rum::http::Server;
use rum::prelude::*;

use rum::controller::auth::{AuthHandler, BasicAuth, SessionAuth};

struct BasicAuthController {
    auth: AuthHandler,
}

impl BasicAuthController {
    pub fn new() -> Self {
        Self {
            auth: AuthHandler::new(BasicAuth {
                user: "admin".to_string(),
                password: "hunter2".to_string(),
            }),
        }
    }
}

#[async_trait]
impl Controller for BasicAuthController {
    fn auth(&self) -> &AuthHandler {
        &self.auth
    }

    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h2>wait, how do you know my pw?</h2>"))
    }
}

#[derive(Default)]
struct LoginController;

#[async_trait]
impl Controller for LoginController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(request.login(1337).redirect("/protected"))
    }
}

struct ProtectedAreaController {
    auth: AuthHandler,
}

impl ProtectedAreaController {
    pub fn new() -> Self {
        Self {
            auth: AuthHandler::new(SessionAuth::default()), // Change this to `SessionAuth::redirect("/login")`
                                                            // to automatically log the user in.
        }
    }
}

#[async_trait]
impl Controller for ProtectedAreaController {
    fn auth(&self) -> &AuthHandler {
        &self.auth
    }

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let session = request.session().unwrap();
        let welcome = format!("<h1>Welcome, user {:?}</h1>", session.session_id);
        Ok(Response::new().html(welcome))
    }
}

#[derive(Default)]
struct LogoutController;

#[async_trait]
impl Controller for LogoutController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(request.logout().redirect("/"))
    }
}

#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Template::cached_static("templates/index.html").await?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Config::load().await?;
    Logger::init();

    Server::new(vec![
        IndexController::default().route("/"),
        BasicAuthController::new().route("/basic"),
        LoginController::default().route("/login"),
        ProtectedAreaController::new().route("/protected"),
        LogoutController::default().route("/logout"),
    ])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}

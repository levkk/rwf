//! Chat controller

use rwf::prelude::*;

#[derive(rwf::macros::PageController)]
#[auth(auth)]
pub struct ChatController {
    auth: AuthHandler,
}

impl ChatController {
    pub fn new() -> Self {
        Self {
            auth: SessionAuth::redirect("/signup").handler(),
        }
    }
}

#[rwf::async_trait]
impl PageController for ChatController {
    async fn get(&self, _request: &Request) -> Result<Response, Error> {
        let rendered = Template::load("templates/chat.html")?.render([("title", "Rwf chat")])?;

        Ok(Response::new().html(rendered))
    }
}

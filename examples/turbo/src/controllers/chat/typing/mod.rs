use rwf::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::User;

#[derive(Default)]
pub struct TypingController;

#[derive(Serialize, Deserialize)]
pub struct TypingState {
    pub typing: bool,
}

#[rwf::async_trait]
impl Controller for TypingController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let state = request.json::<TypingState>()?;

        if let Some(session_id) = request.session_id() {
            if let Some(user_id) = session_id.user_id() {
                let user = {
                    let mut conn = Pool::connection().await?;
                    User::find(user_id).fetch(&mut conn).await?
                };

                let broadcast = Comms::broadcast(&session_id);
                broadcast.send(state.render(&user)?)?;
            }

            Ok(Response::new().json(serde_json::json!({
                "status": "success",
            }))?)
        } else {
            Ok(Response::new().redirect("/signup"))
        }
    }
}

impl TypingState {
    pub fn render(&self, user: &User) -> Result<Message, Error> {
        let typing = Template::load("templates/typing.html")?;
        let rendered = typing.render([("user", user.clone().to_template_value()?)])?;

        let message = if self.typing {
            TurboStream::new(rendered)
                .action("append")
                .target("typing-indicators")
                .render()
        } else {
            TurboStream::new(rendered)
                .action("remove")
                .target(format!("typing-{}", user.id.unwrap()))
                .render()
        };

        Ok(Message::Text(message))
    }
}

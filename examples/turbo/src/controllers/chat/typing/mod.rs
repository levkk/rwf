use rwf::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::User;

#[derive(Default)]
pub struct TypingController;

#[rwf::async_trait]
impl Controller for TypingController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let state = request.json::<TypingState>()?;
        let mut conn = Pool::connection().await?;
        let user = request.user::<User>(&mut conn).await?;

        if let Some(user) = user {
            let broadcast = Comms::broadcast(&user);
            broadcast.send(state.render(request, &user)?)?;

            Ok(serde_json::json!({
                "status": "success",
            })
            .into())
        } else {
            Ok(Response::new().redirect("/signup"))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TypingState {
    pub typing: bool,
}

impl TypingState {
    pub fn render(&self, request: &Request, user: &User) -> Result<TurboStream, Error> {
        let stream = turbo_stream!(
            request,
            "templates/typing.html",
            "typing-indicators"
            "user" => user.clone()
        );

        let message = if self.typing {
            stream.action("append")
        } else {
            stream
                .action("remove")
                .target(format!("typing-{}", user.id.unwrap()))
        };

        Ok(message)
    }
}

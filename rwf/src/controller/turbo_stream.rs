use super::WebsocketController;
use crate::{http::Stream, prelude::*};

#[derive(Default)]
pub struct TurboStream;

#[async_trait]
impl Controller for TurboStream {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        WebsocketController::handle(self, request).await
    }

    async fn handle_stream(&self, request: &Request, stream: Stream<'_>) -> Result<bool, Error> {
        WebsocketController::handle_stream(self, request, stream).await
    }
}

#[async_trait]
impl WebsocketController for TurboStream {}

//! Implement a WebSocket controller to serve Turbo's `<turbo-stream-source>`.
//!
//! ### Example
//!
//! ```rust
//! use rwf::prelude::*;
//! use rwf::controller::TurboStream;
//! use rwf::http::Server;
//!
//! Server::new(vec![
//!     route!("/turbo-stream" => TurboStream),
//! ]);
//! ```
use super::WebsocketController;
use crate::{http::Stream, prelude::*};

/// Turbo Stream WebSocket controller.
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

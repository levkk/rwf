//! Pretty simple HTTP server.
//!
//! Listens for requests and maps them to a handler, if any exists for the specified path.
//! If no handler is matched, return 404 Not Found.
//!
//! The server is using Tokio, so it can support millions of concurrent clients.
use super::{Error, Handler, Request, Response};

use colored::Colorize;
use std::collections::BTreeSet;

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::net::{TcpListener, ToSocketAddrs};
use tracing::{debug, error, info};

/// HTTP server.
pub struct Server {
    handlers: Arc<BTreeSet<Handler>>,
}

impl Server {
    /// Create new HTTP server.
    ///
    /// Accepts a list of handlers.
    // Duplicate handlers are overwritten without warning.
    pub fn new(handlers: Vec<Handler>) -> Self {
        let mut set = BTreeSet::new();
        for handler in handlers {
            set.insert(handler);
        }
        Server {
            handlers: Arc::new(set),
        }
    }

    /// Launch the server.
    pub async fn launch(self, addr: impl ToSocketAddrs) -> Result<(), Error> {
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (mut stream, peer_addr) = listener.accept().await?;
            let handlers = self.handlers.clone();
            let mut found = false;

            tokio::spawn(async move {
                debug!("HTTP new connection from {:?}", peer_addr);

                loop {
                    let request = match Request::read(&mut stream).await {
                        Ok(request) => request,
                        Err(err) => {
                            debug!("client {:?} disconnected: {:?}", peer_addr, err);
                            return;
                        }
                    };

                    let start = Instant::now();

                    for handler in handlers.iter().rev() {
                        // Found matching handler.
                        if request.path().matches(handler.path()) {
                            found = true;

                            // Get response.
                            let response = match handler.handle_internal(&request).await {
                                Ok(response) => response,
                                Err(err) => {
                                    error!(
                                        "{} {}: {:?}",
                                        handler.controller_name().green(),
                                        request.path().path().purple(),
                                        err
                                    );
                                    Response::internal_error(err)
                                }
                            };

                            // Calculate duration.
                            // We include the time to find the handler in the duration.
                            let duration = Instant::now() - start;

                            // Log request.
                            Self::log(&request, handler.controller_name(), &response, duration);

                            // Send reply to client.
                            match response.send(&mut stream).await {
                                Ok(_) => (),
                                Err(err) => {
                                    debug!("{} error {:?}", peer_addr, err);
                                }
                            }

                            break;
                        }
                    }

                    // No handler for this path.
                    if !found {
                        // Log duration of search.
                        let duration = Instant::now() - start;

                        // Generate default not found response.
                        let response = Response::not_found();

                        // Log the response.
                        Self::log(&request, std::any::type_name::<Self>(), &response, duration);

                        // Send reply to client.
                        match response.send(&mut stream).await {
                            Ok(_) => (),
                            Err(err) => {
                                debug!("{} error {:?}", peer_addr, err);
                            }
                        }
                    }
                }
            });
        }
    }

    fn log(request: &Request, controller_name: &str, response: &Response, duration: Duration) {
        info!(
            "{} {} {} ({:.3} ms)",
            controller_name.green(),
            request.path().path().purple(),
            response.status().code(),
            duration.as_secs_f64() * 1000.0,
        );
    }
}

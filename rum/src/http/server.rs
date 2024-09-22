//! Pretty simple HTTP server.
//!
//! Listens for requests and maps them to a handler, if any exists for the specified path.
//! If no handler is matched, return 404 Not Found.
//!
//! The server is using Tokio, so it can support millions of concurrent clients.
use super::{Error, Handler, Request, Response, Router};

use colored::Colorize;
use std::collections::BTreeSet;

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, ToSocketAddrs};
use tracing::{debug, error, info};

/// HTTP server.
pub struct Server {
    handlers: Arc<Router>,
}

impl Server {
    /// Create new HTTP server.
    ///
    /// Accepts a list of handlers.
    // Duplicate handlers are overwritten without warning.
    pub fn new(handlers: Vec<Handler>) -> Self {
        Server {
            handlers: Arc::new(Router::new(handlers).unwrap()),
        }
    }

    /// Launch the server.
    pub async fn launch(self, addr: impl ToSocketAddrs) -> Result<(), Error> {
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let mut stream = BufReader::new(BufWriter::new(stream));
            let handlers = self.handlers.clone();

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

                    match handlers.find(request.path()) {
                        Ok(Some(handler)) => {
                            // Set the matching regex to extract parameters.
                            let request = request.with_params(handler.path_with_regex().params());

                            // Pass the request to the controller to get a response.
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

                            let _ = stream.flush().await;
                        }

                        Ok(None) => {
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

                        Err(err) => {
                            todo!()
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

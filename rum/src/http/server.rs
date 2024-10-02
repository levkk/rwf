//! Pretty simple HTTP server.
//!
//! Listens for requests and maps them to a handler, if any exists for the specified path.
//! If no handler is matched, return 404 Not Found.
//!
//! The server is using Tokio, so it can support millions of concurrent clients.
use super::{Error, Handler, Protocol, Request, Response, Router};

use crate::colors::MaybeColorize;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub enum Stream<'a> {
    Plain(&'a mut BufReader<BufWriter<TcpStream>>),
}

impl<'a> Stream<'a> {
    pub fn stream(&'a mut self) -> impl AsyncRead + AsyncWrite + 'a {
        match self {
            Stream::Plain(stream) => stream,
        }
    }
}

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
            let handlers = self.handlers.clone();

            tokio::spawn(async move {
                match Self::handle_connection(handlers, stream, peer_addr).await {
                    Ok(_) => (),
                    Err(_) => {
                        error!("panic detected, this is a bug; controllers should return an error instead");
                    }
                }
            });
        }
    }

    fn handle_connection(
        handlers: Arc<Router>,
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> JoinHandle<()> {
        let mut stream = BufReader::new(BufWriter::new(stream));

        tokio::spawn(async move {
            debug!("new connection from {:?}", peer_addr);

            loop {
                let request = match Request::read(peer_addr, &mut stream).await {
                    Ok(request) => request,
                    Err(err) => {
                        debug!("client {:?} disconnected: {:?}", peer_addr, err);
                        return;
                    }
                };

                let start = Instant::now();

                match handlers.find(request.path()) {
                    Some(handler) => {
                        // Set the matching regex to extract parameters.
                        let request = request.with_params(handler.path_with_regex().params());

                        // Pass the request to the controller to get a response.
                        let response = match handler.handle_internal(request.clone()).await {
                            Ok(response) => response,
                            Err(err) => {
                                error!(
                                    "{} {} 500 {:?}",
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

                        let websocket_upgrade = response.websocket_upgrade();

                        // Send reply to client.
                        match response.send(&mut stream).await {
                            Ok(_) => (),
                            Err(err) => {
                                debug!("{} error {:?}", peer_addr, err);
                            }
                        }

                        let _ = stream.flush().await;

                        println!("stream starting");

                        match handler.handle_stream(Stream::Plain(&mut stream)).await {
                            Ok(true) => continue,
                            _ => break,
                        };
                    }

                    None => {
                        // Log duration of search.
                        let duration = Instant::now() - start;

                        // Generate default not found response.
                        let response = Response::not_found();

                        // Log the response.
                        Self::log(&request, std::any::type_name::<Self>(), &response, duration);

                        // Send reply to client.
                        match response.send(&mut stream).await {
                            Ok(_) => {
                                let _ = stream.flush().await;
                            }
                            Err(err) => {
                                debug!("2 {} error {:?}", peer_addr, err);
                            }
                        }
                    }
                }
            }
        })
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

//! Pretty simple HTTP server.
//!
//! Listens for requests and maps them to a handler, if any exists for the specified path.
//! If no handler is matched, return 404 Not Found.
//!
//! The server is using Tokio, so it can support millions of concurrent clients.
use super::{Error, Handler, Request, Response, Router};
use crate::controller::Error as ControllerError;

use crate::colors::MaybeColorize;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

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
        info!(
            "Starting {} {} {}",
            "Rum".green(),
            "HTTP".purple(),
            "server".red()
        );

        self.handlers.log_routes();

        let listener = TcpListener::bind(addr).await?;

        info!("Listening on {}", listener.local_addr().unwrap());

        loop {
            select! {
                _ = ctrl_c() => {
                    info!("Shutting down...");
                    return Ok(());
                }

                result = listener.accept()  => {
                    if let Ok((stream, peer_addr)) = result {
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
            }
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
                                error!("{:?}", err);
                                match err {
                                    ControllerError::HttpError(err) => match err.code() {
                                        400 => Response::bad_request(),
                                        403 => Response::forbidden(),
                                        _ => Response::internal_error(err),
                                    },

                                    err => Response::internal_error(err),
                                }
                            }
                        };

                        // Set the session on the request before we pass it down
                        // to the stream handler.
                        let request = request.set_session(response.session().clone());
                        let ok = response.status().ok();

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

                        if stream.flush().await.is_err() {
                            break;
                        }

                        if ok {
                            match handler
                                .handle_stream(&request, Stream::Plain(&mut stream))
                                .await
                            {
                                Ok(true) => continue,
                                _ => break,
                            };
                        }
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
            "{} {} {} {} ({:.3} ms)",
            request.method().to_string().purple(),
            request.path().path().purple(),
            controller_name.green(),
            response.status().code(),
            duration.as_secs_f64() * 1000.0,
        );
    }
}

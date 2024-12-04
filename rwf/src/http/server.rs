//! Simple and performant HTTP server.
//!
//! Listens for requests and maps them to a handler, if any exists for the specified path.
//! If no handler is matched, return `404 - Not Found`.
//!
//! The server is using Tokio and can support millions of concurrent clients.
use super::{Error, Handler, Request, Response, Router};

use crate::colors::MaybeColorize;
use crate::config::get_config;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// Type of TCP connection used by the client.
#[derive(Debug)]
pub enum Stream<'a> {
    /// Plain text (not encrypted).
    Plain(&'a mut BufReader<BufWriter<TcpStream>>),
}

impl<'a> Stream<'a> {
    /// Get the underlying TCP stream reader & writer.
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
    /// Accepts a list of routes and their handlers.
    // Duplicate handlers are overwritten without warning.
    pub fn new(handlers: Vec<Handler>) -> Self {
        Server {
            handlers: Arc::new(Router::new(handlers).unwrap()),
        }
    }

    /// Launch the server. This blocks until the server is shut down (`SIGINT`/Ctrl-C).
    pub async fn launch(self) -> Result<(), Error> {
        let config = get_config();
        let addr = format!("{}:{}", config.general.host, config.general.port);
        info!(
            "Starting {} {} {}",
            "Rwf".green(),
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
            debug!("{} new connection from {:?}", "http".purple(), peer_addr);

            loop {
                let request = match Request::read(peer_addr, &mut stream).await {
                    Ok(request) => request,
                    Err(ref err) => {
                        match err {
                            Error::ContentTooLarge(head) => {
                                let response = Response::content_too_large();
                                let _ = Self::send_response(&mut stream, response).await;

                                info!(
                                    "{} {} {} 413",
                                    head.method().to_string().purple(),
                                    head.path().base().purple(),
                                    std::any::type_name::<Self>().green(),
                                );
                            }

                            _ => (),
                        }
                        debug!(
                            "{} client {:?} disconnected: {}",
                            "http".purple(),
                            peer_addr,
                            err
                        );
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
                                error!("{}", err);
                                Response::internal_error(err)
                            }
                        };

                        // Set the session on the request before we pass it down
                        // to the stream handler.
                        let request = match response.session().clone() {
                            Some(session) => request.set_session(session),
                            None => request,
                        };
                        let ok = response.status().ok();

                        // Calculate duration.
                        // We include the time to find the handler in the duration.
                        let duration = start.elapsed();

                        // Log request.
                        Self::log(&request, handler.controller_name(), &response, duration);

                        if let Err(err) = Self::send_response(&mut stream, response).await {
                            debug!("{} error {:?}", peer_addr, err);
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
                        if let Err(err) = Self::send_response(&mut stream, response).await {
                            debug!("{} error {:?}", peer_addr, err);
                            break;
                        }
                    }
                }
            }
        })
    }

    fn log(request: &Request, controller_name: &str, response: &Response, duration: Duration) {
        let method = request.method().to_string();
        let path = request.path().path();
        let code = response.status().code() as i32;
        let duration = (duration.as_secs_f64() * 1000.0) as f32;

        info!(
            "{} {} {} {} ({:.3} ms)",
            method.purple(),
            path.purple(),
            controller_name.green(),
            code,
            duration,
        );
    }

    async fn send_response(
        mut stream: impl AsyncWrite + Unpin,
        response: Response,
    ) -> Result<(), Error> {
        response.send(&mut stream).await?;
        stream.flush().await?;

        Ok(())
    }
}

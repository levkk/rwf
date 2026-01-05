//! Simple and performant HTTP server.
//!
//! Listens for requests and maps them to a handler, if any exists for the specified path.
//! If no handler is matched, return `404 - Not Found`.
//!
//! The server is using Tokio and can support millions of concurrent clients.

use super::{Error, Handler, Request, Response, Router};
use std::cell::OnceCell;

use crate::colors::MaybeColorize;
use crate::config::get_config;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::task::JoinHandle;
use tokio_rustls::{server::TlsStream, TlsAcceptor};
use tracing::{debug, error, info, warn};

/// Type of TCP connection used by the client.
enum Conn {
    Plain(BufReader<BufWriter<TcpStream>>),
    Tls(BufReader<BufWriter<TlsStream<TcpStream>>>),
}

impl From<TcpStream> for Conn {
    fn from(value: TcpStream) -> Self {
        Self::Plain(BufReader::new(BufWriter::new(value)))
    }
}
impl From<TlsStream<TcpStream>> for Conn {
    fn from(value: TlsStream<TcpStream>) -> Self {
        Self::Tls(BufReader::new(BufWriter::new(value)))
    }
}

#[derive(Debug)]
pub enum Stream<'a> {
    /// Plain text (not encrypted).
    Plain(&'a mut BufReader<BufWriter<TcpStream>>),
    Tls(&'a mut BufReader<BufWriter<TlsStream<TcpStream>>>),
}

impl Conn {
    fn as_stream(&mut self) -> Stream<'_> {
        match self {
            Self::Plain(stream) => Stream::Plain(stream),
            Self::Tls(stream) => Stream::Tls(stream),
        }
    }
}

impl AsyncRead for Conn {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Conn::Plain(s) => Pin::new(s).poll_read(cx, buf),
            Conn::Tls(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Conn {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Conn::Plain(s) => Pin::new(s).poll_write(cx, buf),
            Conn::Tls(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Conn::Plain(s) => Pin::new(s).poll_flush(cx),
            Conn::Tls(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Conn::Plain(s) => Pin::new(s).poll_shutdown(cx),
            Conn::Tls(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}
impl AsyncRead for Stream<'_> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Stream::Plain(s) => Pin::new(s).poll_read(cx, buf),
            Stream::Tls(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream<'_> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Stream::Plain(s) => Pin::new(s).poll_write(cx, buf),
            Stream::Tls(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Stream::Plain(s) => Pin::new(s).poll_flush(cx),
            Stream::Tls(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Stream::Plain(s) => Pin::new(s).poll_shutdown(cx),
            Stream::Tls(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

/*
impl<'a> Stream<'a> {
    /// Get the underlying TCP stream reader & writer.
    pub fn stream(&'a mut self) -> impl AsyncRead + AsyncWrite + 'a {
        match self {
            Stream::Plain(stream) => stream,
            Stream::Tls(stream) => stream.
        }
    }
}*/

/// HTTP server.
pub struct Server {
    handlers: Arc<Router>,
    tls_acceptor: OnceCell<Arc<Option<TlsAcceptor>>>,
}

impl Server {
    /// Create new HTTP server.
    ///
    /// Accepts a list of routes and their handlers.
    // Duplicate handlers are overwritten without warning.
    pub fn new(handlers: Vec<Handler>) -> Self {
        Server {
            handlers: Arc::new(Router::new(handlers).unwrap()),
            tls_acceptor: OnceCell::new(),
        }
    }

    fn tls_config() -> Result<Option<TlsAcceptor>, Error> {
        let config = get_config();
        if let Some(ref cert_file) = config.general.cert_file {
            if let Some(ref key_file) = config.general.key_file {
                let cert =
                    CertificateDer::pem_file_iter(cert_file)?.collect::<Result<Vec<_>, _>>()?;
                let key = PrivateKeyDer::from_pem_file(key_file)?;
                let config = rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(cert, key)?;
                Ok(Some(TlsAcceptor::from(Arc::new(config))))
            } else {
                warn!("Certfile is set but Keyfile is not!");
                Ok(None)
            }
        } else {
            Ok(None)
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
        let _ = self.tls_acceptor.set(Arc::new(Self::tls_config()?));

        info!("Listening on {}", listener.local_addr()?);

        loop {
            select! {
                _ = ctrl_c() => {
                    info!("Shutting down...");
                    return Ok(());
                }

                result = listener.accept()  => {
                    if let Ok((stream, peer_addr)) = result {
                        let handlers = self.handlers.clone();
                        let acceptor = self.tls_acceptor.get().unwrap().clone();
                        tokio::spawn(async move {

                            match Self::handle_connection(handlers, stream, acceptor, peer_addr).await {
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
        tls_acceptor: Arc<Option<TlsAcceptor>>,
        peer_addr: SocketAddr,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut stream = if let Some(acceptor) = tls_acceptor.as_ref() {
                Conn::from(acceptor.accept(stream).await.unwrap())
            } else {
                Conn::from(stream)
            };
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
                            match handler.handle_stream(&request, stream.as_stream()).await {
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

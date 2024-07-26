use super::{Error, Request, Response, super::controller::Route};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use std::future::Future;

pub async fn server<F>(routes: Vec<Route<F>>) -> Result<(), Error>
where
    F: Future<Output = Result<Response, crate::controller::Error>>
{
    let mut listener = TcpListener::bind("0.0.0.0:8000").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;

        let routes = routes.clone();
        tokio::spawn(async move {
            let request = Request::read(&mut stream).await?;
            Response::not_found("not found").send(&mut stream).await?;

            Ok::<(), crate::http::Error>(())
        });
    }

    Ok(())
}

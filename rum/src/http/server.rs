use super::{super::controller::Route, Error, Request, Response};
use std::future::Future;
use tokio::io::{AsyncReadExt};
use tokio::net::{TcpListener};

pub async fn server<F>(_routes: Vec<Route<F>>) -> Result<(), Error>
where
    F: Future<Output = Result<Response, crate::controller::Error>>,
{
    let listener = TcpListener::bind("0.0.0.0:8000").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;

        // let routes = routes.clone();
        tokio::spawn(async move {
            let _request = Request::read(&mut stream).await?;
            Response::not_found("not found").send(&mut stream).await?;

            Ok::<(), crate::http::Error>(())
        });
    }

    Ok(())
}

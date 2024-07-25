use super::{Error, Request, Response};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn server() -> Result<(), Error> {
    let mut listener = TcpListener::bind("0.0.0.0:8000").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;
        let request = Request::read(&mut stream).await?;
        Response::not_found("not found").send(&mut stream).await?;
        println!("request: {:?}", request);
    }

    Ok(())
}

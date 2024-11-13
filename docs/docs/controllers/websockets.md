# WebSockets

Rwf comes with built-in WebSockets support, requiring no additional dependencies or configuration.

## What are WebSockets?

A WebSocket is a bidirectional communication protocol that allows browsers and servers
to talk to each other. Unlike normal HTTP responses,
which are only delivered when the client asks for them, WebSocket messages can be sent by the server at any time.

This is useful for updating web apps in real-time, or sending push notifications when something important
happens on the server, for example.

### How do WebSockets work?

A WebSocket connection is a TCP connection. It's established by sending a regular HTTP request with a special header.
If the server supports WebSockets, like Rwf does, it responds with a special response and upgrades the connection to use
the WebSocket protocol instead of HTTP.

WebSockets allow both clients and servers to send text and binary data, both of which are supported.

## Writing a WebSocket controller

A WebSocket controller is any Rust struct that implements the
[`WebsocketController`](https://docs.rs/rwf/latest/rwf/controller/trait.WebsocketController.html) trait.

The trait has two methods of interest: the first handles new WebSocket connections, and the other
incoming messages from the client.

```rust
use rwf::controller::Websocket;
use rwf::prelude::*;

#[derive(Default, macros::WebsocketController)]
struct Echo;

#[async_trait]
impl WebsocketController for Echo {
    /// Run some code when a new client connects to the WebScoket server.
    async fn handle_connection(
        &self,
        client: &SessionId,
    ) -> Result<(), Error> {
        log::info!("Client {:?} connected to the echo server", client);

        Ok(())
    }

    /// Run some code when a client sends a message to the server.
    async fn handle_message(
        &self,
        client: &SessionId,
        message: Message,
    ) -> Result<(), Error> {
        // Get an app-wide WebSocket channel to the client.
        // This will send a message to the client via WebScoket
        // connection from anywhere in the code.
        let comms = Comms::websocket(client);

        // Send the message back to the client (we're an echo server).
        comms.send(message)?;

        Ok(())
    }
}
```

There are a few things to unpack here. The `handle_message` method is called every time a client sends a message
addressed to this WebSocket controller. What to do with the message depends on the application, but if we
were writing a real-time chat app, we would save it to the database and notify all interested clients of a
new message.

The [`Comms`](https://docs.rs/rwf/latest/rwf/comms/struct.Comms.html) struct is a global data structure that keeps track of who is connected to our server. You can use it
to send a [`Message`](https://docs.rs/rwf/latest/rwf/http/websocket/enum.Message.html) to any client at any time.

!!! note
    The `macros::WebsocketController` automatically implements the `Controller` trait.
    All Rwf controllers have to implement the `Controller` trait, and the `WebsocketController` is no exception.
    The trait automatically implements the `handle` method, however due to the nature of Rust dynamic dispatch,
    the `handle` method of the supertrait has to be called explicitly in the base trait.

    If you were not to use the macro, you could do the same thing manually:

    ```rust
    #[async_trait]
    impl Controller for Echo {
        async fn handle(&self, request: &Request) -> Result<Response, Error> {
            WebsocketController::handle(self, request).await
        }
    }
    ```

## Sending messages to clients

All WebSocket clients have a unique [session](sessions.md) identifier. Sending a message to a client only requires that you know their session ID, which you can obtain from the [`Request`](request.md), for example:

```rust
if let Some(session_id) = request.session_id() {
    let client = Comms::websocket(&session_id);

    client.send("hey there")?;
}
```

WebSocket messages can be delivered to any client from anywhere in the application, including [controllers](index.md) and [background jobs](../background-jobs/index.md).

## Starting a WebSocket server

Since WebSockets are built into Rwf, you can just add the controller to the server at startup:

```rust
use rwf::prelude::*;
use rwf::http::{Server, self};

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    let server = Server::new(vec![
        route!("/websocket" => Echo),
    ])
    .launch("0.0.0.0:8000")
    .await
}
```

### Testing the connection

In a browser of your choice, open up the developer tools console and connect to the WebSocket server:

```javascript
const ws = new WebSocket("ws://localhost:8000/websocket");
```

If everything works, you should see a log line in the terminal where the server is running, indicating a new
client has joined the party.

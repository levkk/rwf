# Turbo Streams

Turbo can process page changes received via a [WebSocket](../../controllers/websockets.md) connection. This enables the server to dynamically update the client's page without the client clicking links or submitting forms. Rwf supports this out of the box.

## WebSocket endpoint

Since Turbo Streams use WebSockets to push changes to the client, you need to create a WebSocket endpoint first. Rwf comes with a controller to do just that without any additional configuration:

```rust
use rwf::prelude::*;
use rwf::http::Server;
use rwf::controllers::TurboStream;

#[tokio::main]
async fn main() {
    Server::new(vec![
        route!("/turbo-stream" => TurboStream),
    ])
    .launch("0.0.0.0:8000")
    .await
    .unwrap()
}
```

## Connect the app

Turbo has a special HTML element which automatically handles WebSocket connections, called `<turbo-stream-source>`. This element needs to specify the WebSocket endpoint and be placed in the body of all pages that wish to support Turbo Streams, for example:

```html
<html>
    <body>
        <turbo-stream-source
            src="ws://localhost:8000/turbo-stream">
        </turbo-stream-source>
    <!-- ... -->
```

WebSocket connections need to specify the absolute URL for the WebSocket server. The endpoint above uses the development server you're running on localhost, but in production this will be different. To make this easier, Rwf comes with a handy template function which figures out which endpoint to use based on your website's URL:

```erb
<html>
    <body>
        <%- rwf_turbo_stream("/turbo-stream") %>
    <!-- ... -->
```

If your website is running on `https://example.com`, this function will create a Turbo Stream connection pointing to `wss://example.com/turbo-stream`.

!!! note
    The `<turbo-stream-source>` element must be placed inside the `<body>`. When visiting pages, Turbo updates the `<body>` element only, while keeping other elements like `<head>` intact. To make sure
    Turbo reconnects to your stream endpoint when loading a page, the stream element needs to be recreated on each page visit.

## Send updates

Updates to pages can be sent from anywhere in the code. The only thing you need is the [session](../../controllers/sessions.md) identifier. If you're sending an update from a [controller](../../controllers/index.md), you can obtain the session ID from the [request](../../controllers/request.md):

```rust
let session_id = request.session_id();
```

Once you have the ID, you can send an update directly to that user:

```rust
use rwf::prelude::*;

// Not all requests will have a session.
if let Some(session_id) = session_id {
    // Create the update.
    let update = TurboStream::new(r#"
        <div id="messages">
            <p>Hi Alice!</p>
            <p>Hello Bob!</p>
        </div>
    "#).action("replace");

    let message = Message::Text(update.render());

    // Send it via a WebSocket connection.
    Comms::websocket(&session_id).send(message)?;
}
```

If you need to send updates to the client from somewhere else besides a controller, e.g. from a [background job](../../background-jobs/index.md), pass the session identifier to that code as an argument. The session identifier is unique and unlikely to change.

## Learn more

- [WebSockets](../../controllers/websockets.md)
- [Template functions](../templates/functions/index.md)
- [Sessions](../../controllers/sessions.md)
- [Hotwired Turbo Streams](https://turbo.hotwired.dev/handbook/streams)

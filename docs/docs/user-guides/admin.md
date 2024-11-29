# Admin panel

Rwf comes with its own admin panel which provides a real time overview of web activity, insights into the background jobs queue, and allows to manipulate database models.

The admin panel is written with Rwf and can be included into any Rwf-powered application.

## Enable the admin panel

The admin panel comes in its own crate: `rwf-admin`. To enable it, add it to your application dependencies:

```bash
cargo add rwf-admin
```

### Preload templates

The admin panel has its own templates and static files which need to be preloaded at runtime:

```rust
use rwf::prelude::*;
use rwf::http::{Server, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Preload templates and static files.
    rwf_admin::install()?;

    // Launch the server below.
}
```

### Add routes

Add the admin panel routes to your HTTP server:

```rust
let mut routes = vec![
    // Your application routes.
];

// Add admin routes.
routes.extend(rwf_admin::routes());

// Launch the server.
Server::new(routes)
    .launch()
    .await?;
```

## Learn more

- [examples/turbo](https://github.com/levkk/rwf/tree/main/examples/turbo) app uses the admin panel
- [rwf-admin](https://github.com/levkk/rwf/tree/main/rwf-admin)

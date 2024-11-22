# Rwf admin

[![Documentation](https://img.shields.io/badge/documentation-blue?style=flat)](https://levkk.github.io/rwf/)
[![Latest crate](https://img.shields.io/crates/v/rwf-admin.svg)](https://crates.io/crates/rwf-admin)
[![Reference docs](https://img.shields.io/docsrs/rwf-admin)](https://docs.rs/rwf/latest/rwf-admin/)

[Rwf](https://crates.io/crates/rwf) admin panel is a web application that provides a real time overview into web activity, background jobs queue insights, and allows to manipulate database models.

The admin panel can run as a standalone application or be integrated into an existing Rwf application.

## Installation

To install Rwf admin panel into your application, you need to add it to your routes and preload its templates at application startup:

```rust
use rwf::prelude::*;
use rwf::http::{Server, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    rwf_admin::install()?;

    let mut routes = vec![];
    // Add your routes...

    routes.extend(rwf_admin::routes());

    Server::new(routes)
        .launch("0.0.0.0:8000")
        .await
}
```

The admin panel is now running on [https://localhost:8000/admin/](https://localhost:8000/admin/).

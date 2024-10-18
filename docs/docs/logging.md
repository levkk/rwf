# Logging

Rwf uses the [`log`](https://docs.rs/log) crate for logging. The crate employs the standard `INFO`, `WARN`, `ERROR`, and `DEBUG` levels to output information of different importance. If you have a logging preference, e.g. you want to use JSON-structured logs without colors, you can use a logging subscriber of your choice. Alternatively, you can use the logger that comes with Rwf, like so:

```rust
use rwf::prelude::*;

#[tokio::main]
async fn main() {
    // Make sure to call this only once in your application.
    Logger::init();

    /* ... */
}
```

## Log queries

By default, queries executed against the database are not logged. If you want to see what's being executed (and how long queries are taking to return results), toggle the `log_queries` setting in the [configuration](../configuration).

## Log requests

All HTTP requests to Rwf are logged at the `INFO` level. This is useful in production to detect application activity and debug any issues (e.g. bad load balancer configuration).

## Default log level

By default, Rwf applications are launched with the `INFO` log level. Since Rwf [`Logger`](https://docs.rs/rwf/latest/rwf/logging/struct.Logger.html) is using [`tracing-subscriber`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/), you can change that by setting the `RUST_LOG` environment variable, for example:

```
export RUST_LOG=debug
```

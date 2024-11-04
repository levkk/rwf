# Local dev overview

Local development with Rwf benefits from the extensive Rust ecosystem, and makes some additions of its own, like [hot reloading](hot-reload.md) of frontend code.

To make your development experience smoother, we recommend you install `cargo-watch` and `cargo-nextest`, like so:

```bash
cargo install cargo-watch cargo-nextest
```

## Watch for changes

`cargo-watch` can monitor your code for changes and restart the server automatically. This makes local development much easier: as you make edits to your code, you don't have to stop and start the server manually:

```bash
cargo watch --exec run
```

### Hot reload

Rwf can refresh pages automatically as they are being changed. If you enable [hot reload](hot-reload.md), and also use `cargo-watch`, make sure to tell it to ignore template changes:

```bash
cargo watch --exec run --ignore "*.html"
```

## Run tests

Running tests with `cargo-nextest` is faster and more ergonomic as opposed to using built-in `cargo test`. If you end up writing tests for your app, you can run them all in parallel:

```
cargo nextest run
```

## Learn more

- [Hot reload](hot-reload.md)

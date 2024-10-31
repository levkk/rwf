# Hot reload

Hot reload, also known as hot module replacement (HMR), is a technique for automatically replacing frontend components that changed during local development, without the developer having to reload the page manually.
While Rwf [templates](../views/templates/index.md) don't use JavaScript frameworks like React or Vue, they do support being reloaded automatically.


## Enable hot reload

To enable template hot reloading, make sure your application is using [Turbo Streams](../views/turbo/streams.md). The page refresh event is delivered from the server using a WebSocket connection.

Current hot reload implementation works best if you are storing your templates in one directory, e.g. `templates`. To enable HMR, launch it before launching the HTTP server:

```rust
use rwf::hmr::hmr;

use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // Enable HMR notifications for any changes
    // to the `templates` directory.
    hmr(PathBuf::from("templates"));

    /* ... */
}
```

When editing templates with your favorite text editor, Rwf will send an event via the Turbo Stream connection which will reload the page every time a template file is saved. Since Turbo makes page reloads seamless, this simulates the behavior of HMR used by frameworks like React or Vue.

### Debug only

HMR only makes sense in development, so the functionality is available in `debug` builds which are used by default when you use `cargo run`. In `release` builds, HMR is disabled.

## Learn more

- [rwf-admin](https://github.com/levkk/rwf/blob/main/rwf-admin/src/main.rs) uses HMR

# Template cache

Templates are compiled and evaluated on the fly. This is handy for local development, allowing you to modify the template without recompiling the Rust app or restarting the web server, but in production could be an unnecessary performance hindrance.

The template cache makes sure a template is compiled only once. All subsequent executions of the template will use an internal representation and are much faster to run.

## Using the cache

To use the template cache, templates must be stored on disk, for example in a `templates` directory. Loading a template should use the [`Template::load`](https://docs.rs/rwf/latest/rwf/view/template/struct.Template.html#method.load) function:

```rust
let template = Template::load("templates/index.html")?;
```

The first time the template is loaded, it will be fetched from disk and compiled. Once compiled, it will be stored in the cache to be reused by all subsequent calls to [`Template::load`](https://docs.rs/rwf/latest/rwf/view/template/struct.Template.html#method.load).

## Enable the cache

The template cache is disabled by default in development, and enabled in production[^1]. To change this behavior, toggle the `cache_templates` setting in [configuration](../../../configuration).

[^1]: This assumes you build your application using the `release` profile, e.g. `cargo build --release`.

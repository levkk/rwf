# Deploying to production

Applications written with Rwf are standard Rust apps that can be deployed to production using existing tools, like buildpacks or Docker. Additionally, Rwf comes with its own CLI that can package your application and run it on bare metal hardware without third-party dependencies.


## Using the CLI

The Rwf CLI can package your application with a single command:

=== "Command"
    ```bash
    rwf-cli package
    ```
=== "Output"
    ```
    $ rwf-cli package
    Finished `release` profile [optimized] target(s) in 0.49s
    packaging binary
    packaging static
    packaging templates
    packaging migrations
    created build.tar.gz
    ```

This will build your application in release mode and bundle the binary, templates, static files and migrations into a single archive called `bundle.tar.gz`. 

Since Rust applications are compiled, they don't require any additional dependencies to run. You can copy the bundle onto your production machine(s), untar it and run the app:

```bash
tar xvf bundle.tar.tz
./app
```

### Cross-compiling

If you're developing on one type of hardware, but your production servers run another, you'll need to compile your application for the right [CPU architecture](https://doc.rust-lang.org/rustc/platform-support.html).

If you have a cross-compiler installed, you can provide the desired architecture as an argument to `rwf-cli`, for example:

```
rwf-cli package --target aarch64-unknown-linux-gnu
```


## Using Docker

Docker can bundle the app and its dependencies together, making sure your application can run anywhere Docker is available.


### Writing a Dockerfile

Writing a Dockerfile for an Rwf application involves compiling the code in release mode and copying over the assets, like static files and templates:

```docker
# Build the app in a separate container.
FROM rust:1-bullseye AS builder
COPY . /build
WORKDIR /build
RUN cargo build --release

# Production container using the same
# Linux distro.
FROM debian:bullseye

# Copy app from build container.
COPY --from=builder /build/target/release/app /app/app

# Copy assets.
COPY templates /app/templates
COPY static /app/static
COPY migrations /app/migrations

# Run the app.
WORKDIR /app
CMD ["app"]
```

Building the application in a separate container makes sure the container running the app in production is small.

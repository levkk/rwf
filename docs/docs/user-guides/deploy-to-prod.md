# Deploying to production

Rwf-powered applications are standard Rust apps which can be deployed to production using existing tools, like buildpacks or Docker. Additionally, Rwf comes with a CLI which can package your applications and run them on bare metal hardware without third-party dependencies.


## Using the CLI

The Rwf CLI can package your application with a single command:

```bash
rwf-cli package
```

This will build your application in release mode and bundle the binary, templates, static files and migrations into a single archive called `bundle.tar.gz`. Since Rust applications are compiled, they don't require any additional dependencies to run.


You can copy the bundle onto your production machine(s), untar it and run the app:

```bash
tar xvf bundle.tar.tz
./app
```

## Using Docker

Docker can run any applications inside a container, bundling the app and its dependencies. This ensures that your app can run anywhere Docker is available.


### Writing a Dockerfile

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

# Copy production config.
COPY rwf.prod.toml /app/rwf.toml

# Run the app.
WORKDIR /app
CMD ["app"]
```

# Build the app in a separate container.
FROM rust:1-bullseye AS builder
COPY . /build
WORKDIR /build
RUN cargo build --release

# This container will run in production, without
# intermediate build artifacts.
FROM debian:bullseye
COPY --from=builder /build/target/release/turbo /app/turbo

# Copy assets
COPY templates /app/templates
COPY migrations /app/migrations
COPY static /app/static

WORKDIR /app
CMD ["turbo"]

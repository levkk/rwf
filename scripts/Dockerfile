FROM rust:1-bullseye AS builder
COPY . /build
WORKDIR /build
RUN cargo build --release

FROM debian:bullseye
COPY --from=builder /build/target/release /app/app
COPY templates /app/templates
COPY migrations /app/migrations
COPY static /app/static
WORKDIR /app
CMD ["app"]

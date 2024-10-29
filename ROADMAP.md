# Features roadmap

Rwf is brand new, but web development is ancient. Many features are missing or are incomplete.

## ORM

- [ ] LEFT JOINs
- [ ] RIGHT JOINs
- [ ] MySQL support
- [ ] SQLite support
- [ ] Distributed locks (with Postgres, not Redis)
- [ ] More tests

## HTTP server

- [ ] HTTP/2, HTTP/3
- [ ] TLS
- [ ] Fuzzy tests (not the cute ones on four legs, the ones that ingest junk into the router and try to make it crash)
- [ ] EventStreams
- [ ] Integration tests
- [ ] Support for multiple WebSocket controllers (`Comms::websocket` assumes only one)
- [ ] Multipart forms

## Dynanic templates

- [x] Better error messages, e.g. syntax errors, undefined variables, functions, etc.
- [ ] More data types support, e.g. UUIDs, timestampts, whatever Rust data types we forgot to add
- [ ] More tests
- [ ] Allow for extending template syntax with user-defined functions (defined at startup)

## Background & scheduled jobs

- [ ] Statistics on running/pending/failed jobs (can be done with VIEWs)
- [ ] More tests
- [ ] Support more crontab syntax extensions
- [ ] More user-friendly (de)ser (https://github.com/levkk/rwf/issues/7)

## Authentication & user sessions

- [ ] Add a default User model, so we can support accounts without any boilerplate (just like Django)
- [ ] OAuth2 (Google/GitHub/etc.) support built-in, user would just need to add key/secret

## Documentation and guides

- [ ] More README-style docs
- [ ] Code docs (rust doc) for every public struct, function, enum, etc.

## Migrate from Python

- [ ] Consider using granian (https://github.com/levkk/rwf/issues/4)

## Migrate from Ruby

- [ ] Add support for running Rake apps (e.g. Rails)

## Built-ins

- [ ] Feature flags and experiments
- [x] Tracking (user requests)

## More?

Please feel free to add more features to this list.

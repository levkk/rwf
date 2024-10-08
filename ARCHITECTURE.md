# Architecture

Rum is built on top of the now ancient [Model-View-Controller (MVC)](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller) architecture. In my experience, this is the only arch that has survived the test of time and scales into millions of lines of code.

## Modules

The code base is split into several modules, described below.

| Module | Description |
| -------|-------------|
| [controller](rum/src/controller) | Controllers, including the standard HTTP `Controller`, RESTful API, and WebSockets. |
| [controller::middleware](rum/src/controller/middleware) | Support for injecting middleware into the HTTP request/response lifecycle. |
| [http](rum/src/http) | Support for HTTP/1.1. Handles URLs, queries, parameters, body, request, response, etc. |
| [http::websocket](run/src/http/websocket) | Support for WebSockets. Implemented from the [MDN spec](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers). |
| [job](rum/src/job) | Background jobs, including scheduled jobs, clock, worker. |
| [model](rum/src/model) | The ORM. Based on a mix between Django and ActiveRecord. |
| [model::pool](rum/src/model/pool) | Database connection pooling. |
| [model::migrations](rum/src/model/migrations) | Support for database schema versioning (migrations). |
| [view](rum/src/view) | Templates inspired by Rails' ERB language. |
| [view::template](rum/src/view/template) | Template language parser and executor. |
| [view::turbo](rum/src/view/turbo) | Integration with Hotwired Turbo. |
| [rum::crypto](rum/src/crypto.rs) | Helpers to encrypt/decrypt data using AES-128. |
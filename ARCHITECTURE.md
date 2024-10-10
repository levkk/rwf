# Architecture

Rwf is built on top of the now ancient [Model-View-Controller (MVC)](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller) architecture. In my experience, this is the only arch that has survived the test of time and scales into millions of lines of code.

This documentation is incomplete. Please feel free to add stuff if you wish.

## Modules

Rwf is split into different modules and source code files. Modules are documented below, in no particular order.

### `controller`

The "C" in MVC, the controller module handles code which is responsible for handling HTTP requests from clients. Three (3) traits are of most interest:

`rwf::controller::Controller`

This trait is required for all controllers wishing to be served by the HTTP server. It only has one required method (`handle`) and every other method is optional and already implemented with defaults. All methods can be overridden, making it very customizable.

`rwf::controller::RestController`

This trait implements the REST framework, specifically the `handle` method of the `Controller` trait, and splits incoming requests into the 6 REST verbs.

`rwf::controller::ModelController`

Same as above, except it also implements all 6 REST verbs using the `Model` type specified. It links directly into the ORM and reads, creates, updates, and deletes the desired records.

`rwf::controller::WebsocketController`

Implements the HTTP -> WebSocket protocol upgrade and communication.

#### `controller::middleware`

This module implements controller middleware. Pretty self-explanatory as soon as you read `mod.rs`.


### `http`

HTTP server and request routing to controllers.

### `job`

Background and scheduled jobs.

### `model`

The ORM and connection pool.

### `view`

Dynamic templates &dash; - our own implementation of basically Rails' ERB.

### `crypto`

Encrypt/decrypt stuff easily with AES-128. I think that [should be enough](https://security.stackexchange.com/questions/14068/why-most-people-use-256-bit-encryption-instead-of-128-bit) but happy to include other ciphers.

### `comms`

Communication between HTTP clients, via Websockets & Tokio `broadcast` module. Helps to connect one part of Rwf with another. In early development still, I'm thinking we could add a lot more things here, e.g. Django-like signals / Rails-like callbacks.
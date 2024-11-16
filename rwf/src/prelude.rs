//! A collection of types, methods and macros
//! which when imported make Rwf development ergonomic and easy.
//!
//! We recommend you import these whenever you work with Rwf primitives:
//!
//! ```
//! use rwf::prelude::*;
//! ```
pub use crate::comms::Comms;
pub use crate::config::Config;
pub use crate::controller::{auth::SessionAuth, AuthHandler};
pub use crate::controller::{
    Authentication, Controller, Error, ModelController, PageController, RestController, SessionId,
};
pub use crate::http::{Cookie, CookieBuilder, Message, Method, Request, Response, ToMessage};
pub use crate::job::{queue_async, queue_delay, Job};
pub use crate::logging::Logger;
pub use crate::model::{Migrations, Model, Pool, Scope, ToSql, ToValue};
pub use crate::view::{Template, ToTemplateValue, TurboStream};

/// A macro to easily implement async traits methods.
pub use async_trait::async_trait;

pub use time::{Duration, OffsetDateTime};
pub use tokio;

pub use macros::{context, crud, engine, render, rest, route, turbo_stream};
pub use rwf_macros as macros;
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;

pub mod error;
pub mod request;
pub mod response;
pub mod route;

pub use error::Error;
pub use request::Request;
pub use response::Response;
pub use route::Route;

use crate::model::{get_connection, Model, Query};
use crate::view::Template;

use std::future::Future;
use tokio::task::JoinHandle;

pub trait Controller<T: Model + Sync> {
   
}

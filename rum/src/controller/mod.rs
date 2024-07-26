pub mod error;
pub mod request;
pub mod response;
pub mod route;

pub use error::Error;
pub use request::Request;
pub use response::Response;
pub use route::Route;

use crate::model::{Model};





pub trait Controller<T: Model + Sync> {}

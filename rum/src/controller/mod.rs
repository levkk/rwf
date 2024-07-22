pub mod error;
pub mod request;
pub mod response;
pub mod route;

pub use error::Error;
pub use request::Request;
pub use response::Response;

use crate::model::{get_connection, Model, Query};

use std::future::Future;

pub trait Controller<T: Model> {
    fn model() -> Query<T>;
    fn index(request: Request) -> impl Future<Output = Result<Response, Error>> {
        async {
            let conn = get_connection().await?;
            let models = Self::model().fetch_all(&conn).await?;
            todo!()
        }
    }

    fn show(request: Request) -> impl Future<Output = Result<Response, Error>>;
    fn create(request: Request) -> impl Future<Output = Result<Response, Error>>;
    fn update(request: Request) -> impl Future<Output = Result<Response, Error>>;
    fn delete(request: Request) -> impl Future<Output = Result<Response, Error>>;
}

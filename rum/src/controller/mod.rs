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
    fn model() -> Query<T> {
        T::all()
    }

    fn controller_name() -> String {
        let struct_name = std::any::type_name::<Self>();
        struct_name
            .split("::")
            .skip(1)
            .collect::<Vec<_>>()
            .join("/")
    }

    fn index(request: Request) -> impl Future<Output = Result<Response, Error>> + Send {
        async move {
            let models = {
                let conn = get_connection().await?;
                Self::model().fetch_all(&conn).await?
            };

            if request.is_json() {
                let mut results = vec![];
                for model in models {
                    results.push(model.to_json()?);
                }
                Response::json(results)
            } else {
                todo!()
            }
        }
    }

    fn show(request: Request) -> impl Future<Output = Result<Response, Error>>;
    fn create(request: Request) -> impl Future<Output = Result<Response, Error>>;
    fn update(request: Request) -> impl Future<Output = Result<Response, Error>>;
    fn delete(request: Request) -> impl Future<Output = Result<Response, Error>>;

    fn handle_index_internal(request: Request) -> JoinHandle<Result<Response, Error>> {
        tokio::spawn(async move { Self::index(request).await })
    }
}

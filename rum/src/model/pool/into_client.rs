use super::{ConnectionGuard, Error, Pool, Transaction};
use std::future::Future;
use tokio_postgres::Client;

use std::ops::Deref;

pub struct Wrapper {
    client: Box<dyn Deref<Target = Client>>,
}

impl Deref for Wrapper {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        self.client.deref()
    }
}

pub trait IntoWrapper {
    fn into_wrapper(self) -> impl Future<Output = Result<Wrapper, Error>>;
}

impl IntoWrapper for ConnectionGuard {
    fn into_wrapper(self) -> impl Future<Output = Result<Wrapper, Error>> {
        async {
            Ok(Wrapper {
                client: Box::new(self),
            })
        }
    }
}

impl IntoWrapper for Pool {
    fn into_wrapper(self) -> impl Future<Output = Result<Wrapper, Error>> {
        async move {
            match self.get().await {
                Ok(guard) => Ok(Wrapper {
                    client: Box::new(guard),
                }),
                Err(e) => Err(e),
            }
        }
    }
}

impl IntoWrapper for Transaction {
    fn into_wrapper(self) -> impl Future<Output = Result<Wrapper, Error>> {
        async {
            Ok(Wrapper {
                client: Box::new(self),
            })
        }
    }
}

// impl IntoWrapper for &Transaction {
// 	fn into_wrapper(self) -> impl Future<Output = Result<Wrapper, Error>> {
// 		async {
// 			Ok(Wrapper {
// 				client: Box::new(self.deref()),
// 			})
// 		}
// 	}
// }

// pub trait IntoClient {
// 	fn into_client(&self) -> impl Future<Output = Result<&Client, Error>>;
// }

// impl<'a> IntoClient for &'a Pool {
// 	fn into_client(&self) -> impl Future<Output = Result<&Client, Error>> {
// 		async {
// 			match self.get().await {
// 				Ok(guard) => Ok(guard),
// 				Err(e) => Err(e),
// 			}
// 		}
// 	}
// }

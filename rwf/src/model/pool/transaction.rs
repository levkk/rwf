//! Manages a transaction lifecycle.
use super::{ConnectionGuard, Error};
use crate::config::get_config;

use std::time::Instant;
use tracing::info;

/// Explicit PostgreSQL transaction.
pub struct Transaction {
    connection: ConnectionGuard,
    savepoints: Vec<String>,
    rollback: bool,
}

impl Transaction {
    /// Start a new transaction on the connection.
    /// The transaction is automatically rolled back if it is not committed
    /// manually using [`Transaction::commit`].
    pub async fn new(mut connection: ConnectionGuard) -> Result<Self, Error> {
        let start = Instant::now();
        connection.query_cached("BEGIN", &[]).await?;

        if get_config().general.log_queries {
            info!("BEGIN ({:.3} ms)", start.elapsed().as_secs_f64() * 1000.0);
        }

        Ok(Self {
            connection,
            savepoints: Vec::with_capacity(16),
            rollback: true,
        })
    }

    /// Commit the transaction to the database.
    /// The connection is automatically returned into the pool.
    pub async fn commit(mut self) -> Result<(), Error> {
        self.rollback = false;

        let start = Instant::now();
        self.connection.query_cached("COMMIT", &[]).await?;

        if get_config().general.log_queries {
            info!("COMMIT ({:.3} ms)", start.elapsed().as_secs_f64() * 1000.0);
        }

        Ok(())
    }

    /// Rollback the transaction.
    /// The connection is automatically returned into the pool.
    pub async fn rollback(mut self) -> Result<(), Error> {
        self.rollback = false;

        let start = Instant::now();
        self.connection.query_cached("ROLLBACK", &[]).await?;

        if get_config().general.log_queries {
            info!(
                "ROLLBACK ({:.3} ms)",
                start.elapsed().as_secs_f64() * 1000.0
            );
        }

        Ok(())
    }

    /// Check if `Transaction` has any savepoints
    /// # Example
    /// ```
    /// use rwf::model::start_transaction;
    /// #[tokio::main]
    /// async fn main() -> Result<(), rwf::model::Error> {
    ///     let mut tx =start_transaction().await?;
    ///     assert!(!tx.has_savepoint());
    ///     tx.savepoint().await?;
    ///     assert!(tx.has_savepoint());
    ///     Ok(())
    /// }
    ///
    /// ```
    pub fn has_savepoint(&self) -> bool {
        !self.savepoints.is_empty()
    }
    /// Create a SAVEPOINT one can rollback to
    /// # Example
    /// ```
    /// use rwf::model::start_transaction;
    /// use rwf::model::prelude::*;
    ///
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct SPTest {
    ///     id: Option<i64>,
    ///     value: String
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), rwf::model::Error> {
    ///     let mut tx =start_transaction().await?;
    ///
    ///     tx.query_cached("CREATE TABLE s_p_tests(id bigserial primary key, value text)", &[]).await?;
    ///     tx.savepoint().await?;
    ///     let _ = SPTest::create(&[("value", "test value")]).fetch(&mut tx).await?;
    ///
    ///     assert_eq!(
    ///         SPTest::all().count(&mut tx).await?,
    ///         1
    ///     );
    ///
    ///     tx.rollback_savepoint().await?;
    ///
    ///    assert_eq!(
    ///         SPTest::all().count(&mut tx).await?,
    ///         0
    ///     );
    ///     tx.rollback().await?;
    ///     Ok(())
    ///}
    /// ```
    pub async fn savepoint(&mut self) -> Result<(), Error> {
        let start = Instant::now();
        let sp = format!("sp{}", self.savepoints.len());
        self.connection
            .query_cached(format!("SAVEPOINT {}", sp).as_str(), &[])
            .await?;
        self.savepoints.push(sp.to_string());
        if get_config().general.log_queries {
            info!(
                "SAVEPOINT {} ({:.3} ms)",
                sp.to_string(),
                start.elapsed().as_secs_f64() * 1000.0
            );
        }
        Ok(())
    }
    /// Release the last SAVEPOINT (if any exists)
    /// Merge all changes between now and its creation into the savepoint prior 9r the transaction (if the current savepoint is the only one)
    /// # Example
    /// ```
    /// use rwf::model::start_transaction;
    /// use rwf::model::prelude::*;
    ///
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct SPTest {
    ///     id: Option<i64>,
    ///     value: String
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), rwf::model::Error> {
    ///     let mut tx =start_transaction().await?;
    ///
    ///     tx.query_cached("CREATE TABLE s_p_tests(id bigserial primary key, value text)", &[]).await?;
    ///     tx.savepoint().await?;
    ///     let _ = SPTest::create(&[("value", "test value")]).fetch(&mut tx).await?;
    ///
    ///     assert_eq!(
    ///         SPTest::all().count(&mut tx).await?,
    ///         1
    ///     );
    ///     tx.savepoint().await?;
    ///     if let Err(_) = SPTest::create(&[("value", "another test value")]).fetch(&mut tx).await {
    ///         tx.rollback_savepoint().await?;
    ///     } else {
    ///         tx.release_savepoint().await?;
    ///     }
    ///
    ///    assert_eq!(
    ///         SPTest::all().count(&mut tx).await?,
    ///         2
    ///     );
    ///     tx.rollback_savepoint().await?;
    ///     assert_eq!(
    ///         SPTest::all().count(&mut tx).await?,
    ///         0
    ///     );
    ///     tx.rollback().await?;
    ///     Ok(())
    ///}
    /// ```
    pub async fn release_savepoint(&mut self) -> Result<(), Error> {
        if !self.has_savepoint() {
            Ok(())
        } else {
            let start = Instant::now();
            let sp = self.savepoints.pop().unwrap();
            self.connection
                .query_cached(format!("RELEASE SAVEPOINT {}", sp).as_str(), &[])
                .await?;
            if get_config().general.log_queries {
                info!(
                    "RELEASE SAVEPOINT {} ({:.3} ms)",
                    sp,
                    start.elapsed().as_secs_f64() * 1000.0
                );
            }
            Ok(())
        }
    }
    /// ROLLBACK to last savepoint if any exists
    /// The savepoint will be removed from `Transaction` thus the user have to issue another `Transaction::savepoint` if another rollback to the same SAVEPOINT may be performed
    /// # Example
    /// ```
    /// use rwf::model::start_transaction;
    /// use rwf::model::prelude::*;
    ///
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct SPTest {
    ///     id: Option<i64>,
    ///     value: String
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), rwf::model::Error> {
    ///     let mut tx =start_transaction().await?;
    ///
    ///     tx.query_cached("CREATE TABLE s_p_tests(id bigserial primary key, value text)", &[]).await?;
    ///     tx.savepoint().await?;
    ///     let _ = SPTest::create(&[("value", "test value")]).fetch(&mut tx).await?;
    ///
    ///     assert_eq!(
    ///         SPTest::all().count(&mut tx).await?,
    ///         1
    ///     );
    ///     tx.savepoint().await?;
    ///     if let Err(_) = SPTest::create(&[("value", 1234)]).fetch(&mut tx).await {
    ///         tx.rollback_savepoint().await?;
    ///     }
    ///
    ///    assert_eq!(
    ///         SPTest::all().count(&mut tx).await?,
    ///         1
    ///     );
    ///     tx.rollback_savepoint().await?;
    ///     assert_eq!(
    ///         SPTest::all().count(&mut tx).await?,
    ///         0
    ///     );
    ///     tx.rollback().await?;
    ///     Ok(())
    ///}
    /// ```
    pub async fn rollback_savepoint(&mut self) -> Result<(), Error> {
        if !self.has_savepoint() {
            Ok(())
        } else {
            let start = Instant::now();
            let sp = self.savepoints.pop().unwrap();
            self.connection
                .query_cached(format!("ROLLBACK TO {}", sp).as_str(), &[])
                .await?;
            if get_config().general.log_queries {
                info!(
                    "ROLLBACK TO {} ({:.3} ms)",
                    sp,
                    start.elapsed().as_secs_f64() * 1000.0
                );
            }
            Ok(())
        }
    }
}

impl Drop for Transaction {
    /// Rollback the transaction and return the connection
    /// to the pool.
    fn drop(&mut self) {
        if self.rollback {
            self.connection.rollback();
        }
    }
}

impl std::ops::Deref for Transaction {
    type Target = ConnectionGuard;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl std::ops::DerefMut for Transaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

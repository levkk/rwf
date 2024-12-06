use super::{Model, Pool, Row, ToValue, Value};
use async_trait::async_trait;
use tokio::task::spawn_blocking;

use thiserror::Error;

/// User model error.
#[derive(Debug, Error)]
pub enum Error {
    /// User already exists.
    #[error("user already exists")]
    UserExists,

    /// User doesn't exist.
    #[error("user does not exist")]
    UserDoesNotExist,

    /// Wrong password.
    #[error("supplied password is incorrect")]
    WrongPassword,

    /// Some database error.
    #[error("{0}")]
    DatabaseError(#[from] super::Error),
}

/// Implement user creation and authentication for any database model
/// which has at least the identifier column and a password column. The identifier
/// column must have a unique index.
#[async_trait]
pub trait UserModel: Model + Sync {
    fn identifier_column() -> &'static str;
    fn password_column() -> &'static str;

    async fn create_user(
        identifier: impl ToValue + Send,
        password: impl ToString + Send,
    ) -> Result<Self, Error> {
        let exists = Self::filter(Self::identifier_column(), identifier.to_value())
            .limit(1)
            .fetch_optional(Pool::pool())
            .await?;

        if exists.is_some() {
            return Err(Error::UserExists);
        }

        let password = password.to_string();

        let password_hash = spawn_blocking(move || crate::crypto::hash(password.as_bytes()))
            .await
            .unwrap()
            .unwrap();

        let user = Self::create(&[
            (Self::identifier_column(), identifier.to_value()),
            (Self::password_column(), password_hash.to_value()),
        ])
        .unique_by(&[Self::identifier_column()])
        .fetch(Pool::pool())
        .await?;

        Ok(user)
    }

    async fn login_user(
        identifier: impl ToValue + Send,
        password: impl ToString + Send,
    ) -> Result<Self, Error> {
        let user = Row::filter(Self::identifier_column(), identifier.to_value())
            .not(Self::password_column(), Value::Null) // Make sure column exists
            .take_one()
            .fetch_optional(Pool::pool())
            .await?;

        if let Some(user) = user {
            let column: String = user.try_get(&Self::password_column()).unwrap();

            let password = password.to_string();

            let valid =
                spawn_blocking(move || crate::crypto::hash_validate(column.as_bytes(), &password))
                    .await
                    .unwrap()
                    .unwrap();

            if valid {
                let row = user.into_inner().unwrap();
                Ok(Self::from_row(row)?)
            } else {
                Err(Error::WrongPassword)
            }
        } else {
            Err(Error::UserDoesNotExist)
        }
    }
}

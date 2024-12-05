// use rwf::model::Error;
use rwf::crypto::{hash, hash_validate};
use rwf::prelude::*;
use tokio::task::spawn_blocking;

pub enum UserLogin {
    NoSuchUser,
    WrongPassword,
    Ok(User),
}

#[derive(Clone, macros::Model)]
pub struct User {
    id: Option<i64>,
    email: String,
    password: String,
    created_at: OffsetDateTime,
}

impl User {
    /// Create new user with email and password.
    pub async fn signup(email: &str, password: &str) -> Result<UserLogin, Error> {
        let hash_password = password.to_owned();
        let encrypted_password = spawn_blocking(move || hash(hash_password.as_bytes()))
            .await
            .unwrap()?;

        match Self::login(email, password).await? {
            UserLogin::Ok(user) => return Ok(UserLogin::Ok(user)),
            UserLogin::WrongPassword => return Ok(UserLogin::WrongPassword),
            _ => (),
        }

        let user = User::create(&[
            ("email", email.to_value()),
            ("password", encrypted_password.to_value()),
        ])
        .fetch(Pool::pool())
        .await?;

        Ok(UserLogin::Ok(user))
    }

    /// Login user with email and password.
    ///
    /// Return a user if one exists and the passwords match.
    /// Return `None` otherwise.
    pub async fn login(email: &str, password: &str) -> Result<UserLogin, Error> {
        if let Some(user) = User::filter("email", email)
            .fetch_optional(Pool::pool())
            .await?
        {
            if hash_validate(password.as_bytes(), &user.password)? {
                return Ok(UserLogin::Ok(user));
            } else {
                return Ok(UserLogin::WrongPassword);
            }
        }

        Ok(UserLogin::NoSuchUser)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_user() {
        Migrations::migrate().await.unwrap();
        let _user = User::signup("test@test.com", "password2").await.unwrap();
        let _user = User::login("test@test.com", "password2").await.unwrap();
    }
}

use rwf::model::{get_connection, migrate};
use rwf::prelude::*;
use rwf::register_callback;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, macros::Model)]
struct User {
    id: Option<i64>,
    mail: String,
    name: String,
}
#[derive(Default)]
struct UserInsertCallback;

#[derive(Default)]
struct UserDeletCallback;
async fn send_mail(addr: String, content: String) {
    // Send Mail function
    // Impplementation just for demonstration purposes.

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    eprintln!("Send Mail {} to {}", content, addr);
    tokio::fs::write(addr, content.as_bytes()).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}

#[async_trait]
impl Callback<User> for UserInsertCallback {
    async fn callback(self, data: User) -> User {
        let mail = format!("Welcome on our Website {}", data.name);
        tokio::spawn(send_mail(data.mail.clone(), mail));
        data
    }
}

#[async_trait]
impl Callback<User> for UserDeletCallback {
    async fn callback(self, data: User) -> User {
        let mail = format!(
            "We're sad that you leave us {}. But your account was deleted! Hope to see you again",
            data.name
        );
        tokio::spawn(send_mail(data.mail.clone(), mail));
        data
    }
}

#[tokio::main]
async fn main() -> Result<(), rwf::http::Error> {
    migrate().await?;
    register_callback!(UserInsertCallback, CallbackKind::Insert);
    register_callback!(UserDeletCallback, CallbackKind::Delete);

    let user = User {
        id: None,
        mail: "test@mail.tld".to_string(),
        name: "Username".to_string(),
    };
    let mut conn = get_connection().await?;

    let user = user.save().fetch(&mut conn).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    assert!(tokio::fs::try_exists(user.mail.as_str()).await?);
    let mail = tokio::fs::read_to_string(user.mail.as_str()).await?;
    assert_eq!(mail, format!("Welcome on our Website {}", user.name));
    tokio::fs::remove_file(user.mail.as_str()).await?;

    let user = user.destroy().fetch(&mut conn).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let mail = tokio::fs::read_to_string(user.mail.as_str()).await?;
    assert!(tokio::fs::try_exists(user.mail.as_str()).await?);
    assert_eq!(
        mail,
        format!(
            "We're sad that you leave us {}. But your account was deleted! Hope to see you again",
            user.name
        )
    );
    tokio::fs::remove_file(user.mail.as_str()).await?;

    Ok(())
}

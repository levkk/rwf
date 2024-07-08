use rum::model::{Model, Pool};
use rum_macros::Model;

#[derive(Clone, Model)]
#[has_many(Order)]
#[allow(dead_code)]
struct User {
    id: i64,
    name: String,
}

#[derive(Clone, Model, Debug)]
#[belongs_to(User)]
#[allow(dead_code)]
struct Order {
    id: i64,
    user_id: i64,
    name: String,
    optional: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Pool::new_local();
    let conn = pool.begin().await?;

    conn.query(
        "CREATE TABLE users (
        id BIGINT NOT NULL,
        name VARCHAR NOT NULL
    )",
        &[],
    )
    .await?;

    conn.query("INSERT INTO users VALUES (2, 'test')", &[])
        .await?;

    conn.query(
        "CREATE TABLE orders (
            id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            name VARCHAR NOT NULL,
            optional VARCHAR
    )",
        &[],
    )
    .await?;

    conn.query("INSERT INTO orders VALUES (1, 2, 'test', 'hello')", &[])
        .await?;

    let order = Order::all()
        .join::<User>()
        .find_by(User::column("id"), 2)
        .fetch(&conn)
        .await?;

    assert_eq!(order.id, 1);
    assert_eq!(order.user_id, 2);
    assert_eq!(order.name, "test");
    assert_eq!(order.optional, Some("hello".to_string()));

    let user = User::all()
        .join::<Order>()
        .find_by("id", 2)
        .fetch(&conn)
        .await?;

    assert_eq!(user.id, 2);
    assert_eq!(user.name, "test");

    conn.rollback().await?;

    Ok(())
}

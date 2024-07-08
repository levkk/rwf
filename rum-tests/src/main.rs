use rum::model::{Model, Pool, Query};
use rum_macros::Model;

use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone, Model)]
#[has_many(Order)]
#[allow(dead_code)]
struct User {
    id: i64,
    name: String,
}

#[derive(Clone, Model, Debug)]
#[belongs_to(User)]
#[has_many(OrderItem)]
#[allow(dead_code)]
struct Order {
    id: i64,
    user_id: i64,
    name: String,
    optional: Option<String>,
}

#[derive(Clone, Model, Debug)]
#[belongs_to(Order)]
#[belongs_to(Product)]
#[allow(dead_code)]
struct OrderItem {
    id: i64,
    order_id: i64,
    product_id: i64,
    amount: f64,
}

#[derive(Clone, Model, Debug)]
#[has_many(OrderItem)]
#[allow(dead_code)]
struct Product {
    id: i64,
    name: String,
}

impl OrderItem {
    fn expensive() -> Query<Self> {
        Self::all().filter("amount", 5.0)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish()
        .init();

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

    conn.query(
        "
        CREATE TABLE order_items (
            id BIGINT NOT NULL,
            order_id BIGINT NOT NULL,
            product_id BIGINT NOT NULL,
            amount DOUBLE PRECISION NOT NULL DEFAULT 5.0
        )
    ",
        &[],
    )
    .await?;

    conn.query(
        "
        CREATE TABLE products (
            id BIGINT NOT NULL,
            name VARCHAR NOT NULL
        )
    ",
        &[],
    )
    .await?;

    conn.query("INSERT INTO orders VALUES (1, 2, 'test', 'hello')", &[])
        .await?;

    conn.query(
        "INSERT INTO order_items VALUES (1, 1, 1, 5.0), (1, 1, 2, 6.0)",
        &[],
    )
    .await?;
    conn.query(
        "INSERT INTO products VALUES (1, 'apples'), (2, 'doodles')",
        &[],
    )
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

    let products = Product::all()
        .join::<OrderItem>()
        .join_nested(OrderItem::join::<Order>().join::<User>())
        .filter(User::column("id"), 2)
        .fetch_all(&conn)
        .await?;
    println!("{:#?}", products);

    conn.rollback().await?;

    Ok(())
}

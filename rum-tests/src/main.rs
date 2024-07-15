use rum::model::{Error, Model, Pool, Query, Scope, ToSql};
use rum::view::template::{Context, Template};
use rum_macros::Model;

use std::time::Instant;
use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone, Model, Debug, PartialEq)]
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
    avg_price: f64,
}

impl OrderItem {
    fn expensive() -> Scope<Self> {
        Self::all().filter_gt("amount", 5.0)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
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
            name VARCHAR NOT NULL,
            avg_price DOUBLE PRECISION NOT NULL DEFAULT 5.0
        )
    ",
        &[],
    )
    .await?;

    conn.query("INSERT INTO orders VALUES (1, 2, 'test', 'optional')", &[])
        .await?;

    conn.query(
        "INSERT INTO order_items VALUES (1, 1, 1, 5.0), (1, 1, 2, 6.0)",
        &[],
    )
    .await?;
    conn.query(
        "INSERT INTO products VALUES (1, 'apples', 6.0), (2, 'doodles', 7.0)",
        &[],
    )
    .await?;

    let mut order = Order::all()
        .join::<User>()
        .find_by(User::column("id"), 2)
        .fetch(&conn)
        .await?;

    assert_eq!(order.id(), 1);
    assert_eq!(order.user_id, 2);
    assert_eq!(order.name, "test");
    assert_eq!(order.optional, Some("optional".to_string()));

    order.name = "test 2".into();
    let order = order.save().fetch(&conn).await?;
    assert_eq!(order.name, "test 2");

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

    let mut product = products.first().unwrap().clone();
    product.name = "something else".to_string();

    let product = product.save().fetch(&conn).await?;
    assert_eq!(product.name, "something else");
    println!("{:#?}", product);

    let order_items = OrderItem::expensive()
        .join::<Order>()
        .filter(Order::column("user_id"), 2)
        .fetch_all(&conn)
        .await?;

    println!("{:?}", order_items);

    let user = User::lock()
        .filter("id", 6)
        .or(|query| query.filter("id", 2).filter("name", "test"))
        .first_one()
        .fetch(&conn)
        .await?;

    println!("{:?}", user);

    let user = User::find([1, 2].as_slice()).fetch_all(&conn).await?;
    assert_eq!(user.clone().pop().unwrap().id, 2);

    assert!(User::find(3).fetch(&conn).await.is_err());

    println!("{:?}", user);

    let exists = User::all()
        .filter("id", 2)
        .filter("name", "test")
        .order("count")
        .exists(&conn)
        .await?;

    assert_eq!(exists, true);

    let count = User::all().filter("id", 2).count(&conn).await?;

    assert_eq!(count, 1);

    let raw = User::find_by_sql("SELECT * FROM users LIMIT 1")
        .fetch(&conn)
        .await?;
    assert_eq!(raw.id, 2);

    conn.rollback().await?;

    let template = Template::new("templates/test.html").await?;
    let mut context = Context::default();
    context.set("title", "hello")?;
    context.set("description", "world")?;
    context.set("vars", vec!["hello", "world"])?;
    let start = Instant::now();
    let result = template.render(&context)?;
    println!("{}, elapsed: {}", result, start.elapsed().as_secs_f64());

    Ok(())
}

#![allow(dead_code)]
use rum::model::{Model, Pool, Scope};
use rum::view::{template::Template, Templates};
use rum::{
    controller::{
        middleware::{Middleware, RateLimiter, SecureId},
        AllowAll, AuthHandler, MiddlewareSet, SessionId, StaticFiles, WebsocketController,
    },
    http::{websocket, Request, Response, Stream},
    job::Job,
    model::{migrate, rollback},
    serde::{Deserialize, Serialize},
    Controller, Error, ModelController, RestController, Server,
};
use rum_macros::{Context, Model};

use std::time::Instant;
use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};

mod components;
mod controllers;
mod models;

#[derive(Clone, Model, Debug, PartialEq)]
#[has_many(Order)]
#[allow(dead_code)]
struct User {
    id: Option<i64>,
    name: String,
}

#[derive(Clone, Model, Debug, Serialize, Deserialize)]
#[belongs_to(User)]
#[has_many(OrderItem)]
#[allow(dead_code)]
struct Order {
    id: Option<i64>,
    user_id: i64,
    name: String,
    optional: Option<String>,
}

#[derive(Clone, Model, Debug)]
#[belongs_to(Order)]
#[belongs_to(Product)]
#[allow(dead_code)]
struct OrderItem {
    id: Option<i64>,
    order_id: i64,
    product_id: i64,
    amount: f64,
}

#[derive(Clone, Model, Debug)]
#[has_many(OrderItem)]
#[allow(dead_code)]
struct Product {
    id: Option<i64>,
    name: String,
    avg_price: f64,
}

impl OrderItem {
    fn expensive() -> Scope<Self> {
        Self::all().filter_gt("amount", 5.0)
    }
}

struct BaseController {
    id: String,
}

#[rum::async_trait]
impl Controller for BaseController {
    async fn handle(&self, request: &Request) -> Result<Response, rum::controller::Error> {
        RestController::handle(self, request).await
    }
}

#[rum::async_trait]
impl RestController for BaseController {
    type Resource = String;

    async fn get(
        &self,
        _request: &Request,
        id: &String,
    ) -> Result<Response, rum::controller::Error> {
        Ok(Response::new().html(format!("<h1>controller id: {}, id: {}</h1>", self.id, id)))
    }
}

struct BasePlayerController {}

#[rum::async_trait]
impl Controller for BasePlayerController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        RestController::handle(self, request).await
    }
}

#[rum::async_trait]
impl RestController for BasePlayerController {
    type Resource = i64;

    async fn get(&self, request: &Request, id: &i64) -> Result<Response, Error> {
        if let Some(session) = request.session() {
            session
                .websocket()
                .send(websocket::Message::Text("controller websocket".into()))?;
        }
        Ok(Response::new().html(format!("<h1>base player controller, id: {}</h1>", id)))
    }

    async fn list(&self, _request: &Request) -> Result<Response, Error> {
        // match tokio::fs::File::create("fsdf").await {
        //     Ok(_) => (),
        //     Err(err) => error!(err),
        // };
        Ok(Response::new().html("list all the players"))
    }
}

struct OrdersController {
    auth: AuthHandler,
    middlware: MiddlewareSet,
}

#[rum::async_trait]
impl Controller for OrdersController {
    fn auth(&self) -> &AuthHandler {
        &self.auth
    }

    fn middleware(&self) -> &MiddlewareSet {
        &self.middlware
    }

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        ModelController::handle(self, request).await
    }
}

#[rum::async_trait]
impl RestController for OrdersController {
    type Resource = i64;
}

#[rum::async_trait]
impl ModelController for OrdersController {
    type Model = Order;
}

struct JobOne;

#[rum::async_trait]
impl Job for JobOne {
    async fn execute(&self, _args: serde_json::Value) -> Result<(), rum::job::Error> {
        Ok(())
    }
}

struct JobTwo;

#[rum::async_trait]
impl Job for JobTwo {
    async fn execute(&self, args: serde_json::Value) -> Result<(), rum::job::Error> {
        println!("job two args: {:?}", args);
        Err(rum::job::Error::Unknown("random error".to_string()))
    }
}

struct MyWebsocketController {}

impl MyWebsocketController {
    pub fn new() -> Self {
        MyWebsocketController {}
    }
}

#[rum::async_trait]
impl Controller for MyWebsocketController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        WebsocketController::handle(self, request).await
    }

    async fn handle_stream(&self, request: &Request, stream: Stream<'_>) -> Result<bool, Error> {
        WebsocketController::handle_stream(self, request, stream).await
    }
}

#[rum::async_trait]
impl WebsocketController for MyWebsocketController {
    async fn client_message(
        &self,
        user_id: &SessionId,
        message: websocket::Message,
    ) -> Result<(), Error> {
        println!("echo: {:?}", message);
        // send it back
        let comms = rum::comms::get_comms();
        let sender = comms.websocket_sender(user_id);
        let _ = sender.send(message);
        Ok(())
    }
}

struct IndexController;

#[rum::async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, rum::controller::Error> {
        Ok(Template::cached_static("templates/index.html").await?)
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

    rollback().await?;
    migrate().await?;

    let pool = Pool::new_local();
    let mut conn = pool.get().await?;

    conn.client()
        .query(
            "INSERT INTO orders (user_id, name, optional) VALUES (2, 'test', 'optional')",
            &[],
        )
        .await?;

    conn.client().query(
        "INSERT INTO order_items (order_id, product_id, amount) VALUES (1, 1, 5.0), (1, 2, 6.0)",
        &[],
    )
    .await?;
    conn.client()
        .query(
            "INSERT INTO products (name, avg_price) VALUES ('apples', 6.0), ('doodles', 7.0)",
            &[],
        )
        .await?;

    let mut order = Order::all()
        .join::<User>()
        .find_by(User::column("id"), 2)
        .fetch(&mut conn)
        .await?;

    assert_eq!(order.id, Some(1));
    assert_eq!(order.user_id, 2);
    assert_eq!(order.name, "test");
    assert_eq!(order.optional, Some("optional".to_string()));

    order.name = "test 2".into();
    let order = order.save().fetch(&mut conn).await?;
    assert_eq!(order.name, "test 2");

    let user = User::all()
        .join::<Order>()
        .find_by("id", 2)
        .fetch(&mut conn)
        .await?;

    assert_eq!(user.id, Some(2));
    assert_eq!(user.name, "test");

    let products = Product::all()
        .join::<OrderItem>()
        .join_nested(OrderItem::join::<Order>().join::<User>())
        .filter(User::column("id"), 2)
        .fetch_all(&mut conn)
        .await?;
    println!("{:#?}", products);

    let mut product = products.first().unwrap().clone();
    product.name = "something else".to_string();

    let product = product.save().fetch(&mut conn).await?;
    assert_eq!(product.name, "something else");
    println!("{:#?}", product);

    let order_items = OrderItem::expensive()
        .join::<Order>()
        .filter(Order::column("user_id"), 2)
        .fetch_all(&mut conn)
        .await?;

    println!("{:?}", order_items);

    let user = User::lock()
        .filter("id", 6_i64)
        .or(|query| query.filter("id", 2).filter("name", "test"))
        .first_one()
        .fetch(&mut conn)
        .await?;

    println!("{:?}", user);

    let user = User::find([1, 2].as_slice()).fetch_all(&mut conn).await?;
    assert_eq!(user.clone().pop().unwrap().id, Some(2));

    assert!(User::find(3).fetch(&mut conn).await.is_err());

    println!("{:?}", user);

    let exists = User::all()
        .filter("id", 2_i64)
        .filter("name", "test")
        .order("count")
        .exists(&mut conn)
        .await?;

    assert_eq!(exists, true);

    let count = User::all().filter("id", 2).count(&mut conn).await?;

    assert_eq!(count, 1);

    let raw = User::find_by_sql("SELECT * FROM users LIMIT 1", &[])
        .fetch(&mut conn)
        .await?;
    assert_eq!(raw.id, Some(2));

    let product = Product {
        id: None,
        avg_price: 56.0,
        name: "test 2".to_string(),
    };

    let product = product.save().fetch(&mut conn).await?;

    // conn.rollback().await?;

    #[derive(Context)]
    struct MyContext {
        title: String,
        description: String,
        vars: Vec<String>,
        product: Product,
        products: Vec<Product>,
    }

    let context = MyContext {
        title: "hello".to_string(),
        description: "world".into(),
        vars: vec!["hello".into(), "world".into()],
        product: product.clone(),
        products: vec![product.clone()],
    };

    let template = Templates::cache().await.get("templates/test.html").await?;
    let start = Instant::now();
    let result = template.render(&context.try_into()?)?;
    println!(
        "{}, elapsed: {}",
        result,
        start.elapsed().as_secs_f64() * 1000.0
    );

    JobOne {}
        .execute_async(serde_json::json!({
            "arg1": 2,
        }))
        .await?;

    JobTwo {}
        .execute_async(serde_json::json!({
            "arg2": 1,
        }))
        .await?;

    // Worker::new(vec![JobOne {}.job(), JobTwo {}.job()])
    //     .start()
    //     .await?
    //     .spawn();
    println!("number: {}", rum::crypto::encrypt_number(1).unwrap());

    Server::new(vec![
        StaticFiles::serve("static")?,
        IndexController {}.route("/"),
        MyWebsocketController::new().route("/websocket"),
        BaseController {
            id: "5".to_string(),
        }
        .route("/base"),
        BasePlayerController {}.route("/base/player"),
        OrdersController {
            // auth: AuthHandler::new(BasicAuth {
            //     user: "test".to_string(),
            //     password: "test".to_string(),
            // }),
            auth: AuthHandler::new(AllowAll {}),
            middlware: MiddlewareSet::new(vec![
                RateLimiter::per_second(10).middleware(),
                SecureId {}.middleware(),
            ]),
        }
        .route("/orders"),
    ])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}

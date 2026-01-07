#![allow(dead_code)]
use rwf::crypto::encrypt_number;
use rwf::model::{Column, Model, Pool, Value};
use rwf::view::Templates;
use rwf::{
    controller::{
        middleware::{Middleware, RateLimiter, SecureId},
        AllowAll, AuthHandler, MiddlewareSet, SessionId, StaticFiles, WebsocketController,
    },
    http::{Request, Response, Server, Stream},
    job::Job,
    model::{
        callbacks::{Callback, CallbackKind},
        migrate, rollback,
    },
    prelude::*,
    register_callback,
};
use rwf_macros::Context;
use std::str::FromStr;

use std::time::Instant;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_redoc::Redoc;

mod components;
mod controllers;
pub mod models;

use crate::models::oapi_backend::OpenApiDocs;
use crate::models::CreateUserCallback;
use crate::OpenApiTargets::Rapidoc;
use models::{Order, OrderItem, Product, User};

//[derive(OpenApi)]
#[derive(Clone, OpenApi, Default, Copy)]
#[openapi(
    info(
        title="rwf", version="0.2.1", contact(name="levkk", url="https://github.com/levkk?tab=packages", email="none@cf.org"), license(name="MIT", url="https://github.com/levkk/rwf/blob/main/LICENSE"),
        description = "OpenAPI definitions and informations about the RWF crate. Also provides API/Type descriptions for Models / ModelController"
    ),
    external_docs(url = "https://rustwebframework.org/", description="Getting started Dcumentation"),
    tags(
        (name = "Model", description = "MOdel/ModdelController related Enndpoints"),
    (name = "RWF", description = "The OpenAPI Specs of RWF. Metainformations about RWF and the guys working on. Informations about anything what have to do whith an impplementation has a different Tag", external_docs(url="https://github.com/levkk/rwf/", description="Link to the git, as any Question could find a answer there")),
    ),
)]
pub struct OpenApiController;

///pub struct RwfOpenApi;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Hash)]
enum OpenApiTargets {
    Yaml,
    Json,
    Doc,
    Redoc,
    Rapidoc,
    Swagger,
    Deny,
}

impl FromStr for OpenApiTargets {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yaml" => Ok(OpenApiTargets::Yaml),
            "json" => Ok(OpenApiTargets::Json),
            "doc" => Ok(OpenApiTargets::Doc),
            "redoc" => Ok(OpenApiTargets::Redoc),
            "rapoidoc" => Ok(OpenApiTargets::Rapidoc),
            "swagger" => Ok(OpenApiTargets::Swagger),
            "deny" => Ok(OpenApiTargets::Deny),
            _ => Err("Unknown OpenApi Targets"),
        }
    }
}

impl std::fmt::Display for OpenApiTargets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenApiTargets::Yaml => write!(f, "yaml"),
            OpenApiTargets::Json => write!(f, "json"),
            OpenApiTargets::Doc => write!(f, "doc"),
            OpenApiTargets::Redoc => write!(f, "redoc"),
            OpenApiTargets::Rapidoc => write!(f, "rapoidoc"),
            OpenApiTargets::Swagger => write!(f, "swagger"),
            OpenApiTargets::Deny => write!(f, "deny"),
        }
    }
}

impl OpenApiController {
    fn match_url(self, request: &Request) -> OpenApiTargets {
        return match request.path().path() {
            "/doc/openapi.json" => OpenApiTargets::Json,
            "/doc/openapi.yaml" => OpenApiTargets::Yaml,
            "/doc" => OpenApiTargets::Doc,
            "/doc/redoc" => OpenApiTargets::Redoc,
            "/doc/rapidoc" => OpenApiTargets::Rapidoc,
            "/doc/swagger" => OpenApiTargets::Swagger,
            "/doc/openapi" => match request.query().get::<OpenApiTargets>("target") {
                Some(target) => match target {
                    OpenApiTargets::Yaml => OpenApiTargets::Yaml,
                    OpenApiTargets::Json => OpenApiTargets::Json,
                    OpenApiTargets::Doc => OpenApiTargets::Doc,
                    OpenApiTargets::Redoc => OpenApiTargets::Redoc,
                    OpenApiTargets::Rapidoc => OpenApiTargets::Rapidoc,
                    OpenApiTargets::Swagger => OpenApiTargets::Swagger,
                    OpenApiTargets::Deny => OpenApiTargets::Deny,
                },
                None => OpenApiTargets::Deny,
            },
            _ => OpenApiTargets::Deny,
        };
    }
    fn match_header(&self, request: &Request) -> OpenApiTargets {
        match request.header("accept").map(|s| s.as_str()) {
            Some("application/json") => OpenApiTargets::Json,
            Some("application/yaml") => OpenApiTargets::Yaml,
            None => OpenApiTargets::Json,
            Some(_) => OpenApiTargets::Deny,
        }
    }

    pub fn nest_model_appis() {
        //toipa::openapi::OpenApi {
        let rwfapi = Self::openapi()
            .nest("/api/", models::oapi_backend::OpenApiDocs::openapi())
            .nest("/api/users", models::oapi_users::ApiDoc::openapi())
            .nest("/api/orditems", models::oapi_order_items::ApiDoc::openapi())
            .nest("/api/products", models::oapi_produucts::ApiDoc::openapi());
        eprintln!("{}", serde_json::to_string_pretty(&rwfapi).unwrap());
    }
}

#[async_trait]
impl Controller for OpenApiController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let target = self.match_url(request);
        let target = if OpenApiTargets::Deny.eq(&target) {
            self.match_header(request)
        } else {
            target
        };
        let oapi = OpenApiDocs::openapi()
            .nest("/users", models::oapi_users::ApiDoc::openapi())
            .nest("/products", models::oapi_produucts::ApiDoc::openapi())
            .nest("/orderItems", models::oapi_order_items::ApiDoc::openapi()); //.nest("/tmodel", crate::models::oapi_test_model::ApiDoc::openapi());
        let redoc = Redoc::new(oapi.clone());
        let rapidoc = utoipa_rapidoc::RapiDoc::new(include_str!("./controllers/rapidoc.json"));

        return match target {
            OpenApiTargets::Yaml => {
                Ok(Response::new().text(oapi.to_yaml().map_err(|e| Error::Error(Box::new(e)))?))
            }
            OpenApiTargets::Json => Ok(Response::new().json(oapi)?),
            OpenApiTargets::Doc => Ok(Response::bad_request()),
            OpenApiTargets::Redoc => Ok(Response::new().html(redoc.to_html())),
            OpenApiTargets::Rapidoc => Ok(Response::new().html(rapidoc.to_html())),
            OpenApiTargets::Swagger => Ok(Response::new().html(redoc.to_html())),
            OpenApiTargets::Deny => Ok(Response::forbidden()),
        };
    }
}

struct BaseController {
    id: String,
}

#[async_trait]
impl Controller for BaseController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        RestController::handle(self, request).await
    }
}

#[async_trait]
impl RestController for BaseController {
    type Resource = String;

    async fn get(&self, _request: &Request, id: &String) -> Result<Response, Error> {
        Ok(Response::new().html(format!("<h1>controller id: {}, id: {}</h1>", self.id, id)))
    }
}

struct BasePlayerController {}

#[async_trait]
impl Controller for BasePlayerController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        RestController::handle(self, request).await
    }
}

#[async_trait]
impl RestController for BasePlayerController {
    type Resource = i64;

    async fn list(&self, _request: &Request) -> Result<Response, Error> {
        // match tokio::fs::File::create("fsdf").await {
        //     Ok(_) => (),
        //     Err(err) => error!(err),
        // };
        Ok(Response::new().html("list all the players"))
    }

    async fn get(&self, request: &Request, id: &i64) -> Result<Response, Error> {
        request
            .session()
            .websocket()
            .send(Message::Text("controller websocket".into()))?;
        Ok(Response::new().html(format!("<h1>base player controller, id: {}</h1>", id)))
    }
}

struct OrdersController {
    auth: AuthHandler,
    middlware: MiddlewareSet,
}

#[async_trait]
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

#[async_trait]
impl RestController for OrdersController {
    type Resource = i64;
}

#[async_trait]
impl ModelController for OrdersController {
    type Model = Order;
}

struct JobOne;

#[async_trait]
impl Job for JobOne {
    async fn execute(&self, _args: serde_json::Value) -> Result<(), rwf::job::Error> {
        Ok(())
    }
}

struct JobTwo;

#[async_trait]
impl Job for JobTwo {
    async fn execute(&self, args: serde_json::Value) -> Result<(), rwf::job::Error> {
        println!("job two args: {:?}", args);
        Err(rwf::job::Error::Unknown("random error".to_string()))
    }
}

struct MyWebsocketController {}

impl MyWebsocketController {
    pub fn new() -> Self {
        MyWebsocketController {}
    }
}

#[async_trait]
impl Controller for MyWebsocketController {
    async fn handle_stream(&self, request: &Request, stream: Stream<'_>) -> Result<bool, Error> {
        WebsocketController::handle_stream(self, request, stream).await
    }

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        WebsocketController::handle(self, request).await
    }
}

#[async_trait]
impl WebsocketController for MyWebsocketController {
    async fn client_message(&self, user_id: &SessionId, message: Message) -> Result<(), Error> {
        println!("echo: {:?}", message);
        // send it back
        use rwf::comms::Comms;
        let sender = Comms::websocket(user_id);
        let _ = sender.send(message);
        Ok(())
    }
}

struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let encs = (1..29)
            .into_iter()
            .map(|i| encrypt_number(i).unwrap())
            .collect::<Vec<String>>();
        render!(request, "templates/index.html", "encs" => encs)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    register_callback!(CreateUserCallback, CallbackKind::Insert);

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

    let pool = Pool::from_env();
    let mut conn = pool.get().await?;

    let user1 = User::create(&[("id", 31.to_value()), ("name", "callback".to_value())])
        .fetch(&mut conn)
        .await?;
    let user2 = User::create(&[("id", 32.to_value()), ("name", "callback2".to_value())])
        .fetch(&mut conn)
        .await?;

    let delq = User::all().filter_gt("id", 30.to_value()).delete();

    let mut del_cnt = delq.fetch_all(&mut conn).await?;
    del_cnt.sort_by(|u1, u2| u1.id.cmp(&u2.id));
    assert_eq!(del_cnt, vec![user1, user2]);

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

    let template = Templates::cache().get("templates/test.html")?;
    let start = Instant::now();
    let result = template.render(&context)?;
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
    println!("number: {}", encrypt_number(1)?);

    let user_no_order = User::all()
        .join_left::<Order>()
        .filter(Column::new("orders", "id"), Value::Null)
        .fetch(&mut conn)
        .await;
    assert!(user_no_order.is_ok());
    let user_no_order = user_no_order?;
    assert_eq!(user_no_order.name, "noorder".to_string());

    let view = Order::all()
        .select_columns(Order::all_columns().as_slice())
        .join::<User>()
        .select_aggregated(&[(Column::new("users", "name"), "", Some("username"))]);
    let order_data = view.fetch_picked(&mut conn).await?;
    assert_eq!(
        order_data.get_entry("name").unwrap().1,
        &Value::String("test".to_string())
    );
    assert_eq!(
        order_data.get_entry("username").unwrap().1,
        &Value::String("test".to_string())
    );

    let ordctl = OrdersController {
        auth: AuthHandler::new(AllowAll {}),
        middlware: MiddlewareSet::new(vec![
            RateLimiter::per_second(10).middleware(),
            SecureId::default().middleware(),
        ]),
    };
    let hdl = ordctl.crud("/orders");
    let _path = hdl.path().clone();

    //ApiDoc::openapi().paths
    use rwf_admin::*;
    use utoipa_redoc;
    install()?;
    let engine = engine().auth(AuthHandler::new(rwf::controller::auth::BasicAuth {
        user: "rwf_admin".to_string(),
        password: "SPbgE5uipuPr7BVDXjifOFqdlQxVVPi".to_string(),
    }));

    let static_files = StaticFiles::new("static")?
        .preload("/static/pre", b"Hello World!")
        .preload(
            "/static/redoc",
            utoipa_redoc::Redoc::new("http://127.0.0.1:8000/doc/openapi.json")
                .to_html()
                .as_bytes(),
        );

    Server::new(vec![
        static_files.handler(),
        IndexController {}.route("/"),
        MyWebsocketController::new().route("/websocket"),
        BaseController {
            id: "5".to_string(),
        }
        .route("/base"),
        BasePlayerController {}.route("/base/player"),
        OpenApiController::default().wildcard("/doc"),
        engine!("/admin" => engine),
        rwf_admin::static_files()?,
    ])
    .launch()
    .await?;

    Ok(())
}

#![allow(dead_code)]
use rwf::crypto::encrypt_number;
use rwf::model::{get_connection, Column, Model, Pool, Value};
use rwf::view::Templates;
use rwf::{
    controller::{AuthHandler, SessionId, StaticFiles, WebsocketController},
    http::{Request, Response, Server, Stream},
    job::Job,
    model::{
        callbacks::{Callback, CallbackKind},
        migrate, rollback,
    },
    prelude::*,
    register_callback,
};
use rwf_macros::{generate_openapi_model_controller, Context};
use std::time::Instant;
use tracing_subscriber::{filter::LevelFilter, fmt, util::SubscriberInitExt, EnvFilter};

mod components;
mod controllers;
pub mod models;

use crate::models::CreateUserCallback;
use models::{Order, OrderItem, Product, User};

use rwf::controller::middleware::SecureId;
use rwf::controller::oidc::{OidcUser as OUser, *};
use rwf::controller::{
    AllowAll, BasicAuth, Middleware, MiddlewareSet, OpenApiController, RateLimiter,
};
#[derive(Debug, Clone, Serialize, Deserialize, macros::Model, ToSchema, ToResponse)]
struct OidcUser {
    #[schema(minimum = 1, example = 128, format = "Int16")]
    id: Option<i16>,
    #[schema(format = "uuid", example = "750590ca-ee80-11f0-92ed-2218e57cbd42")]
    sub: Uuid,
    #[schema(example = "john")]
    name: String,
    #[schema(format = "email", example = "john@mail.tld")]
    email: String,
    #[schema(
        format = "password",
        example = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICIzSi13U0FRSWNscThXSkxLdEY1NzIxVWY3ZVp5VEZBV3JUdzVtZXFwTnZjIn0.eyJleHAiOjE3NjgwOTI2OTMsImlhdCI6MTc2ODA5MjM5MywiYXV0aF90aW1lIjoxNzY4MDkyMzkyLCJqdGkiOiJvZnJ0YWM6MmEwZWMyMDAtMWM2Ni0zMzg1LTk5N2ItNDNkNmYyOGE4MTgxIiwiaXNzIjoiaHR0cHM6Ly9zc28uemV1c3JzLm9yZy9yZWFsbXMvb2lkYyIsImF1ZCI6ImFjY291bnQiLCJzdWIiOiI5MzgyZjE5Zi0yYWNjLTQ2ODctYmI4ZC1lNWFkYTliMTdlZWIiLCJ0eXAiOiJCZWFyZXIiLCJhenAiOiJyd2YiLCJzaWQiOiI5ZDQ5MjE5Zi1mZDFmLTc5NjAtYzk4Yi1kNWFlYWYxZDI2NGUiLCJhY3IiOiIxIiwiYWxsb3dlZC1vcmlnaW5zIjpbImh0dHA6Ly8xMjcuMC4wLjE6ODAwMCJdLCJyZWFsbV9hY2Nlc3MiOnsicm9sZXMiOlsib2ZmbGluZV9hY2Nlc3MiLCJ1bWFfYXV0aG9yaXphdGlvbiIsImRlZmF1bHQtcm9sZXMtb2lkYyJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgb2ZmbGluZV9hY2Nlc3MiLCJlbWFpbF92ZXJpZmllZCI6ZmFsc2UsIm5hbWUiOiJTeXN0ZW0gQWRtaW5pc3RyYXRvciIsInByZWZlcnJlZF91c2VybmFtZSI6InN5c2FkbSIsImdpdmVuX25hbWUiOiJTeXN0ZW0iLCJmYW1pbHlfbmFtZSI6IkFkbWluaXN0cmF0b3IiLCJlbWFpbCI6ImFkbWluQHpldXNycy5vcmcifQ.W3fsv2Jf_xhdK061zf2qyW--8bGXLuy-j51Fyti1JcJjV2OYcYlN4uhonH1jlw532dY_7z3_HtosxmLB84tB52JBfm5st3aRmOQG1kuUN23H08AWxoDRr1Ik9e57Wl3gnsc_3grDZgBrGh47YYDZO50U0qC3fF4Dlsp3kFgFhMGOEswNHveC-ic6APJakZH0hgH0UWge5oBJuluHMhrC7nE-pLX7b5M2r_RtcdMmNJlHmidZS4McNjfgH1ShD4bH01WfuYWUKYS5skt_Hx1ga1Eo8F2NpnbwGOSk75jin36luzlA4QpDMhweoqq0I7R1ynEPnP9YGMneAkrnLMhuog"
    )]
    access: String,
    #[schema(
        format = "password",
        example = "eyJhbGciOiJIUzUxMiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJkNmY4YWFmYi1lODQ2LTRmZGItOGU0MC01ZDQxOTY5ZDlkNzQifQ.eyJpYXQiOjE3NjgwOTIzOTMsImp0aSI6Ijc2OTJmYjYxLWViYWYtNjk1YS00YjEwLTA0NzQ1ZjJmODEyYiIsImlzcyI6Imh0dHBzOi8vc3NvLnpldXNycy5vcmcvcmVhbG1zL29pZGMiLCJhdWQiOiJodHRwczovL3Nzby56ZXVzcnMub3JnL3JlYWxtcy9vaWRjIiwic3ViIjoiOTM4MmYxOWYtMmFjYy00Njg3LWJiOGQtZTVhZGE5YjE3ZWViIiwidHlwIjoiT2ZmbGluZSIsImF6cCI6InJ3ZiIsInNpZCI6IjlkNDkyMTlmLWZkMWYtNzk2MC1jOThiLWQ1YWVhZjFkMjY0ZSIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgc2VydmljZV9hY2NvdW50IHJvbGVzIHdlYi1vcmlnaW5zIG9mZmxpbmVfYWNjZXNzIGJhc2ljIGFjciJ9.d0nNel7jTwdQtPRob7Ekq1x5MvWkN3iRCDelw3pTxqH3ZmXPiCT3r024WXrZeswgN7nEdCBvnwDEtgHhV8CuTQ"
    )]
    refresh: String,
    #[schema(format = "datetime", example = "2026-01-11 0:51:33.31944813 +00:00:00")]
    expire: OffsetDateTime,
}

#[generate_openapi_model_controller(i16, OidcUser)]
#[derive(macros::ModelController)]
#[middleware(middleware)]
struct OidcUserController {
    middleware: MiddlewareSet,
}

impl Default for OidcUserController {
    fn default() -> Self {
        Self {
            middleware: MiddlewareSet::new(vec![SecureId::default().middleware()]),
        }
    }
}

#[async_trait]
impl OUser for OidcUser {
    async fn from_token(
        token: StandardTokenResponse<CoreIdTokenFields, CoreTokenType>,
        userinfo: UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim>,
    ) -> Result<Self, rwf::model::Error> {
        let name = userinfo
            .standard_claims()
            .preferred_username()
            .unwrap()
            .to_string();
        let sub = uuid::Uuid::parse_str(userinfo.standard_claims().subject().to_string().as_str())
            .unwrap();
        let email = userinfo.standard_claims().email().unwrap().to_string();
        let access = token.access_token().secret().clone();
        let refresh = token.refresh_token().unwrap().secret().clone();
        let expire = OffsetDateTime::now_utc()
            .checked_add(Duration::nanoseconds(
                token.expires_in().unwrap().as_nanos() as i64,
            ))
            .unwrap();

        let mut conn = get_connection().await?;
        OidcUser::find_or_create_by(&[
            ("name", name.to_value()),
            ("sub", sub.to_value()),
            ("email", email.to_value()),
            ("access", access.to_value()),
            ("refresh", refresh.to_value()),
            ("expire", expire.to_value()),
        ])
        .unique_by(&["sub"])
        .fetch(&mut conn)
        .await
    }
    fn access_token(&self) -> AccessToken {
        AccessToken::new(self.access.clone())
    }
    fn refresh_token(&self) -> RefreshToken {
        RefreshToken::new(self.refresh.clone())
    }
    fn expire(&self) -> &OffsetDateTime {
        &self.expire
    }
    fn update_token(
        mut self,
        token: StandardTokenResponse<CoreIdTokenFields, CoreTokenType>,
    ) -> Self {
        self.access = token.access_token().secret().clone();
        self.refresh = token.refresh_token().unwrap().secret().clone();
        self.expire = OffsetDateTime::now_utc()
            .checked_add(Duration::nanoseconds(
                token.expires_in().unwrap().as_nanos() as i64,
            ))
            .unwrap();
        self
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

#[generate_openapi_model_controller(i64, User)]
#[derive(Clone, macros::ModelController)]
#[auth(auth)]
struct UserController {
    auth: AuthHandler,
}
impl Default for UserController {
    fn default() -> Self {
        Self {
            auth: BasicAuth {
                user: "test".to_string(),
                password: "".to_string(),
            }
            .handler(),
        }
    }
}

#[generate_openapi_model_controller(i64, Order)]
#[derive(Clone, macros::ModelController)]
#[auth(auth)]
#[middleware(middleware)]
#[skip_csrf]
struct OrderController {
    auth: AuthHandler,
    middleware: MiddlewareSet,
}
impl Default for OrderController {
    fn default() -> Self {
        //let auth: OidcAuthentication<OidcUser> = OidcAuthentication::default();
        Self {
            auth: AllowAll.handler(),
            middleware: MiddlewareSet::new(vec![
                RateLimiter::per_second(10).middleware(),
                SecureId::default().middleware(),
            ]),
        }
    }
}
#[generate_openapi_model_controller(i64, Product)]
#[derive(Clone, macros::ModelController)]
#[auth(auth)]
struct ProductController {
    auth: AuthHandler,
}
impl Default for ProductController {
    fn default() -> Self {
        Self {
            auth: OidcAuthentication::<OidcUser>::default().handler(),
        }
    }
}
#[generate_openapi_model_controller(i64, OrderItem)]
#[derive(Clone, macros::ModelController)]
#[auth(auth)]
struct OrderItemController {
    auth: AuthHandler,
}
impl Default for OrderItemController {
    fn default() -> Self {
        Self {
            auth: OidcAuthentication::<OidcUser>::default().handler(),
        }
    }
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

    assert!(exists);

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
    /*
        let ordctl = OrdersController {
            auth: AuthHandler::new(AllowAll {}),
            ,
        };
        let hdl = ordctl.crud("/orders");
        let _path = hdl.path().clone();
    */
    use rwf_admin::*;

    install()?;
    let engine = engine().auth(AuthHandler::new(BasicAuth {
        user: "rwf_admin".to_string(),
        password: "SPbgE5uipuPr7BVDXjifOFqdlQxVVPi".to_string(),
    }));

    let static_files = StaticFiles::new("static")?.preload("/static/pre", b"Hello World!");

    Server::new(vec![
        static_files.handler(),
        IndexController {}.route("/"),
        MyWebsocketController::new().route("/websocket"),
        BaseController {
            id: "5".to_string(),
        }
        .route("/base"),
        BasePlayerController {}.route("/base/player"),
        route!("/openapi" => OpenApiController),
        engine!("/admin" => engine),
        rwf_admin::static_files()?,
        crud!("/api/users" => UserController),
        crud!("/api/products" => ProductController),
        crud!("/api/orders" => OrderController),
        crud!("/api/orderitems" => OrderItemController),
        crud!("/users" => OidcUserController),
        route!("/oidc" => OidcController::<OidcUser>),
    ])
    .launch()
    .await?;

    Ok(())
}

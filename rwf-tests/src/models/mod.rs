use rwf::async_trait;
use rwf::controller::oidc::{OidcUser as OUser, *};
use rwf::model::callbacks::Callback;
use rwf::model::{get_connection, Model, Scope, ToValue};
use rwf::prelude::{utoipa, Deserialize, Serialize, ToResponse, ToSchema};
use rwf_macros::Model;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Clone, Model, Debug, PartialEq, Serialize, Deserialize, ToSchema, ToResponse)]
#[has_many(Order)]
#[allow(dead_code)]
#[schema(
    title = "An simple User.",
    description = "A User/Customer implementation, which is referenced by Order"
)]
#[response(description = "Representation of a single User azzoziated with the his Orders")]
pub struct User {
    #[schema(minimum = 1, format = "Int64", example = 512)]
    pub(crate) id: Option<i64>,
    #[schema(required = true, examples("John", "Maria"))]
    pub(crate) name: String,
}

#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[belongs_to(Order)]
#[belongs_to(Product)]
#[allow(dead_code)]
pub struct OrderItem {
    #[schema(minimum = 1, example = 128, format = "Int64")]
    pub(crate) id: Option<i64>,
    pub(crate) order_id: i64,
    pub(crate) product_id: i64,
    amount: f64,
}

#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[has_many(OrderItem)]
#[allow(dead_code)]
pub struct Product {
    pub(crate) id: Option<i64>,
    pub(crate) name: String,
    pub(crate) avg_price: f64,
}

#[derive(Debug, Default)]
pub struct CreateUserCallback;

#[async_trait]
impl Callback<User> for CreateUserCallback {
    async fn callback(mut self, data: User) -> User {
        eprintln!("{:?}", data);
        data
    }
}
#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[belongs_to(User)]
#[has_many(OrderItem)]
#[allow(dead_code)]
#[schema(
    title = "An exaple Order",
    description = "A Order in the DB System. Referebces the user who made the order and is refereenced by all related order items",
    examples(json!({"user_id": 64, "name": "Test Order"}))
)]
#[response(
    description = "Rerpresentation of a single Order azzoziated with the buying User and ordere3d Items",
    examples(("InsertResponse" = (summary = "Response of a minimal Insert", value = json!({"id": 128, "user_id": 64, "name": "Test Order"}))))
)]
pub struct Order {
    #[schema(minimum = 1, example = 128, format = "Int64")]
    pub(crate) id: Option<i64>,
    #[schema(minimum = 1, example = 32, format = "Int64")]
    pub(crate) user_id: i64,
    pub(crate) name: String,
    #[schema(required = false, nullable = true)]
    pub(crate) optional: Option<String>,
}

impl OrderItem {
    pub fn expensive() -> Scope<Self> {
        Self::all().filter_gt("amount", 5.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Model, ToSchema, ToResponse)]
pub struct OidcUser {
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

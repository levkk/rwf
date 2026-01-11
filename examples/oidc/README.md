# OIDC

Rwf comes with an OIDC Integration. One only need to implement a User struct with the `OidcUser` trait implemented.

```rust
#[derive(macros::Model, Clone, Serialize, Deserialize)]
struct User {
    id: Option<i64>,
    name: String,
    email: String,
    token: serde_json::Value,
    expire: OffsetDateTime,
}

#[async_trait]
impl OidcUser for User {
    async fn from_token(
        token: StandardTokenResponse<CoreIdTokenFields, CoreTokenType>,
        userinfo: UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim>,
    ) -> Result<Self, rwf::model::Error> {
        let name = userinfo
            .standard_claims()
            .preferred_username()
            .unwrap()
            .to_string();
        let email = userinfo.standard_claims().email().unwrap().to_string();
        let expire = OffsetDateTime::now_utc()
            .checked_add(Duration::nanoseconds(
                token.expires_in().unwrap().as_nanos() as i64,
            ))
            .unwrap();
        let token = serde_json::json!({"access": token.access_token(), "refresh": token.refresh_token().unwrap()});
        let mut conn = get_connection().await?;
        User::create(&[
            ("name", name.to_value()),
            ("email", email.to_value()),
            ("expire", expire.to_value()),
            ("token", token.to_value()),
        ])
            .unique_by(&["email"])
            .fetch(&mut conn)
            .await
    }

    fn access_token(&self) -> AccessToken {
        serde_json::from_value(self.token.get("access").unwrap().clone()).unwrap()
    }

    fn refresh_token(&self) -> RefreshToken {
        serde_json::from_value(self.token.get("refresh").unwrap().clone()).unwrap()
    }

    fn expire(&self) -> &OffsetDateTime {
        &self.expire
    }

    fn update_token(
        mut self,
        token: StandardTokenResponse<CoreIdTokenFields, CoreTokenType>,
    ) -> Self {
        self.token = serde_json::json!({"access": token.access_token(), "refresh": token.refresh_token().unwrap()});
        self
    }
}
```

## Login

To protect a Route with the OIDC Login, one have to set the `OidcAuthentication` handler as the `AuthHandler` for Controller.
```rust
struct TestController {
    auth: AuthHandler,
}

impl Default for TestController {
    fn default() -> Self {
        Self {
            auth: OidcAuthentication::<User>::default().handler(),
        }
    }
}

#[async_trait]
impl Controller for TestController {
    fn auth(&self) -> &AuthHandler {
        &self.auth
    }

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Respnse::not_implemented())    
    }
}
```

## OIDC Response Handler

The AuthHandler is only responsible for issue a Request to the OIDC Provider. A Endpoint to handle the OIDC Response is required also.

```rust
#[tokio::main]
async fn main() -> Result<(), rwf::http::Error> {
    migrate().await?;
    rwf::http::Server::new(vec![
        route!("/" => TestController),
        route!("/oidc" => OidcController::<User>),
    ])
    .launch()
    .await
}
```
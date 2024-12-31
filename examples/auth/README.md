
# Authentication & sessions

Rwf has a customizable authentication and authorization system. All HTTP requests can be checked against some conditions, e.g. a header or a cookie value, and allowed access to a controller. If authorization fails, a default HTTP response, like a redirect or a `401 - Unauthorized` can be returned.

## Included authentication

Rwf comes with three built-in authentication mechanisms:

1. Basic HTTP authentication
2. Token-based authentication (incl. bearer tokens)
3. Session authentication

### Enabling authentication

The default behavior for all controllers is to allow all requests. To enable authentication, implement the `auth` method when defining a controller:

```rust
use rwf::controller::auth::{BasicAuth, AuthHandler}

struct ProtectedController {
    auth: AuthHandler,
}

impl ProtectedController {
    fn new() -> ProtectedController {
        Self {
            auth: AuthHandler::new(BasicAuth {
                user: "admin".to_string(),
                password: "super-secret".to_string(),
            })
        }
    }
}

#[rwf::async_trait]
impl Controller for ProtectedController {
    /// Specify the authentication handler for this controller.
    fn auth(&self) -> &AuthHandler {
        &self.auth
    }

    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().text("Welcome to the protected area!"));
    }
}
```

When a browser opens a page served by this controller, a user/password prompt will have to be filled to see the protected page.

### Session authentication

Rwf implements its own user sessions. They are stored in a cookie, and encrypted with AES-128. The user can't see or modify the contents of the cookie, so arbitrary data can be stored in it to identify the user securely.

To enable session authentication, specify the `SessionAuth` handler in the controller:

```rust
use rwf::controller::auth::SessionAuth;

impl ProtectedController {
    fn new() -> ProtectedController {
        Self {
            auth: AuthHandler::new(SessionAuth::redirect("/login"))
        }
    }
}
```

When users visit a page served by this controller, they will be redirected to `/login` URL if they don't have a session or if their session has expired.

#### Session validity

By default, sessions are valid for 4 days. This setting is [configurable](#configuration). If a user requests a page with a valid session, Rwf will automatically renew the session for another session validity period; this ensures your active users don't get logged out.

#### Anonymous sessions

All requests to a Rwf server are provided with a session. If the user is not logged in, the session is anonymous. This ensures that all requests are authenticated to a browser, which enables features like WebSockets and request tracking. Anonymous sessions are not allowed to access controllers protected by session authentication.

#### Logging in users

To login a user, call the `login` method on the request:

```rust
struct LoginController;

#[rwf::async_trait]
impl Controller for LoginController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let user_id = 1234; // You can get this from the database,
                            // if you have a users table, for example.

        let response = request.login(user_id);

        Ok(response)
    }
}
```

You can safely store the primary key of your users table in the session since the session is encrypted. The browser can't see this value, only the Rwf server can.

#### Logging out users

Users are automatically logged out after a period of inactivity (configurable, see [session validity](#session-validity)). Alternatively, you can call the `logout` method on the request
and return the response:

```rust
async fn handle(&self, request: &Request) -> Result<Response, Error> {
    let response = request.logout();
    Ok(response)
}
```

### Implementing your own authentication

Rwf authentication is fully customizable. You can design your own authentication mechanism by implementing the `Authentication` trait:

```rust
use rwf::controller::auth::Authentication;

#[derive(Default)]
struct NoWorkSundays;

#[rwf::async_trait]
impl Authentication for NoWorkSundays {
    /// Return true if request is allowed, false to deny it.
    async fn authorize(&self, request: &Request) -> Result<bool, Error> {
        let now = OffsetDateTime::now_utc();

        let bypass = request.headers().get("X-I-Need-To-Work-Today").is_some();

        // Allow access on all days except Sunday.
        Ok(now.day() != 0 || bypass)
    }

    /// Optional access denied response.
    /// The default is 401 - Unauthorized
    async fn denied(&self) -> Result<Response, Error> {
        Ok(Response::redirect("https://www.nps.gov"))
    }
}
```

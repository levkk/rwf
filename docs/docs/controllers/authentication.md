# Authentication

Rwf has multiple authentication and authorization mechanisms. Different kinds of authentication require their own kinds of user-supplied credentials. The most commonly used mechanism is [Session](sessions.md) authentication, which has built-in methods for easy use in [controllers](index.md).

## Session authentication

[Session](sessions.md) authentication checks that the user-supplied session cookie is valid (not expired) and contains an authenticated session. If that's not the case, the request is either rejected with a `401 - Unauthorized` or provided an endpoint to re-authenticate, e.g., using a username and password, with a `302 - Found` redirect.

### Enable session authentication

To enable session authentication, it needs to be configured on the controller by implementing the [`auth`](https://docs.rs/rwf/latest/rwf/controller/trait.Controller.html#method.auth) method:

```rust
use rwf::prelude::*;

/// A controller that requires authentication.
struct Private {
    auth: AuthHandler,
}

impl Default for Private {
    fn default() -> Self {
        Private {
            // Redirect unauthenitcated requests to the `/login` route.
            auth: AuthHandler::new(
                SessionAuth::redirect("/login"),
            ),
        }
    }
}

#[async_trait]
impl Controller for Private {
    /// Enable authentication on this controller.
    fn auth(&self) -> &AuthHandler {
        &self.auth
    }

    /* ... */
}
```

## Basic authentication

HTTP Basic is a form of authentication using a global username and password. It's not particularly secure, but it's good enough to protect an endpoint quickly against random visitors. Enabling basic authentication is as simple
as setting an [`AuthHandler`](https://docs.rs/rwf/latest/rwf/controller/auth/struct.AuthHandler.html) with [`BasicAuth`](https://docs.rs/rwf/latest/rwf/controller/auth/struct.BasicAuth.html) on your [controller](index.md). See [examples/auth](https://github.com/levkk/rwf/tree/main/examples/auth) for examples on how to do this.

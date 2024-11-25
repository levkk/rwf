# Building pages

Typical navigation around a web app consist of `GET` requests that retrieve pages (generated with links or buttons) , and `POST` requests which submit new information and do something useful with it (using forms). `GET` and `POST` are the basic building blocks of any web app, and Rwf makes it easy to build pages with it.

## Page controller

The basic [`Controller`](index.md) doesn't disambiguate between `GET` and `POST` requests. If used for every page in your app, you end up having to write boilerplate code like this:

```rust
if request.get() {
    // Retrive page
} else if request.post() {
    // Handle form submission
} else {
    return Ok(Response::not_allowed());
}
```

To avoid doing this and cluttering your codebase, Rwf comes with the [`PageController`](https://docs.rs/rwf/latest/rwf/controller/trait.PageController.html). This controller trait implements the `GET`/`POST` split automatically and routes requests to two separate methods: `async fn get` and `async fn post`.

Let's use the example of a login page built using the `PageController`:

```rust
use rwf::prelude::*;

#[derive(Default, macros::PageController)]
struct Login;

impl PageController for Login {
    // Handle GET and show the login form.
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        render!(request, "templates/login.html")
    }

    // Handle POST, receive form data, check information, and
    // redirect the logged in user to a different page.
    async fn post(&self, request: &Request) -> Result<Response, Error> {
        let form = request.form_data();

        let email = form.get_required::<String>("email");
        let password = form.get_required::<String>("password");

        // Check that the user exists

        Ok(Response::new().login(user.id).redirect("/account"))
    }
}
```

The `async fn get` method renders the login form, while the `async fn post` takes care of the actual login process by handling the form submission via `POST`.

Once you register this controller with the server, one route will handle `GET` and `POST` requests:

```rust
Server::new(vec![route!("/login" => Login)])
    .launch("0.0.0.0:8000")
    .await
```

#### Note on macros

!!! note
    If you're not particularly interested in how Rwf works under the hood, you can skip this section (at least for now).

You may have noticed the odd `macros::PageController` macro in the login controller declaration. This macro expands to this:

```rust
impl Controller for Login {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        PageController::handle(self, request).await
    }
}
```

Rwf HTTP server can only serve controllers that implement the `Controller` trait, so all supertraits must
implement it as well. `PageController` is a supertrait of `Controller` that actually implements the required `async fn handle` method from `Controller`, but due to how dynamic dispatch works in Rust, it has to be called manually from the "child"[^1] trait.

[^1]: Rust traits don't really work like child/parent classes in object-oriented programming, hence the "air" quotes.

To avoid boilerplate code, the `macros::PageController` does this automatically.

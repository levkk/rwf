# Cookies

HTTP cookies are a special header which contains key/value-encoded information. Cookies are set on the server typically, and the client (like a browser) should store them on their end, and send them with each
subsequent request to the server.

Cookies allow to persist information between, what are otherwise, stateless HTTP requests.

## Read cookies

Cookies sent by the browser can be read inside a [controller](../) by calling the [`cookies`](https://docs.rs/rwf/latest/rwf/http/request/struct.Request.html#method.cookies) method:

```rust
let cookies = request.cookies();
```

Since cookies are encoded as key/value pairs, fetching a cookie value can be done by knowing its name:

```rust
let session_id = cookies.get("session_id");

if let Some(session_id) = session_id {
    println!("session_id: {}", session_id.value());
}
```

More often than not, cookies are used to store plain text information, so no special decoding procedure is required to read the cookie value.

## Set cookies

Setting cookies on the server can be done when crafting a [response](../response):

```rust
use rwf::prelude::*;

let mut response = Response::new();

let cookie = CookieBuilder::new()
    .name("session_id")
    .value("1234")
    .max_age(Duration::days(1))
    .build();

response
    .cookies()
    .add(cookie);
```

This produces a `Set-Cookie` header encoded with the cookie name, value, and other attributes like `MaxAge`. You can learn more about cookie attributes and their meaning on [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie).

## Private cookies

Private cookies are cookies which have been encrypted, so the client can't see their contents, or modify them without the server detecting (and automatically rejecting) them.
They are useful for storing sensitive information like a user's session, which can be used in later requests to authenticate requests.

### Set private cookies

Setting private cookies on the response works much like regular cookies, except instead of using `add`, you need to use [`add_private`](https://docs.rs/rwf/latest/rwf/http/cookies/struct.Cookies.html#method.add_private):

```rust
response
    .cookies()
    .add_private(cookie)?;
```

Cookies are [encrypted](../../encryption) with AES-128, using the security key set in the [configuration](../../configuration).


### Read private cookies

Reading private cookies works much like regular cookies, except instead of using `get`, you need to use [`get_private`](https://docs.rs/rwf/latest/rwf/http/cookies/struct.Cookies.html#method.get_private):

```rust
let session_id = cookies.get_private("session_id")?;
```

Decryption will be done automatically, and the controller will be able to accees the plain text value of the cookie.

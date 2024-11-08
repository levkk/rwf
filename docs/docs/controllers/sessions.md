# Sessions

A session is an [encrypted](../security/encryption.md) [cookie](cookies.md) managed by Rwf. It contains a unique identifier for each browser using your web app. All standard-compliant browsers connecting to Rwf-powered apps will have a Rwf session set automatically, and should send it back on each request.

## Session types

Rwf has two kind of sessions: guest sessions and authenticated sessions. Guest sessions have a random alphanumeric identifier, while user sessions have a number identifier, meant to refer to a unique user ID in your database.

When using sessions, you can distinguish between the two like so:

```rust
match request.session_id() {
    SessionId::Guest(id) => { /* handle guest session */ }
    SessionId::Authenticated(user_id) => { /* handle user session */ }
}
```


### Authenticate user

To give a user an authenticated session, i.e. log them into your app, you can set the session cookie with the user ID on the response:

```rust
async fn handle(&self, request: &Request) -> Result<Response, Error> {
    let response = request.login(1234);
    Ok(response)
}
```

## Check for valid session

All [controllers](index.md) can check for the presence of a valid session:

```rust
let session = request.session();

let valid = session
    .map(|session| !session.expired())
    .unwrap_or(false);
```

Unless the session cookie is set and has been encrypted using the correct algorithm and secret key, calling [`session`](https://docs.rs/rwf/latest/rwf/http/request/struct.Request.html#method.session) will return `None`.

#### Expired sessions
If the session is expired, it's advisable not to trust its point of origin. While the contents are guaranteed to be accurate, the browser sending the data has not been validated in several weeks (4 weeks, by default).

### Session authentication

Rwf can ensure all requests have valid and current (not expired) sessions. To enable this feature, enable the [`SessionAuth`](https://docs.rs/rwf/latest/rwf/controller/auth/struct.SessionAuth.html) [authentication](authentication.md) on your controllers. Guest sessions will be refused access, while authenticated sessions will be allowed through.

## Store data in session

Rwf sessions allow you to privately store arbitrary JSON-encoded data. Since browsers place limits on cookie sizes, this data should be relatively small. To store some data in the session, you can set it on the [response](response.md):

```rust
let session = Session::new(
    serde_json::json!({
        "data": "secret_value"
    })
);

let response = Response::new()
  .set_session(session);
```

## Renew sessions

Sessions are automatically renewed on each request. This allows your active users to remain "logged in", while inactive ones would be redirected to a login page if session [authentication](authentication.md) is enabled.

Expired sessions are not renewed, so a user holding an expired session will need to use an authentication controller to get a new valid session.

# Sessions

A session is an [encrypted](../../encryption) [cookie](../cookies) managed by Rwf. It contains a unique identifier for each browser using your web app. All standard-compliant browsers talking to Rwf-powered apps will have a Rwf session set, and should send it back on each request.

## Check for valid session

All [controllers](../) can check for the presence of a valid session:

```rust
let session = request.session();

let valid = session
    .map(|session| !session.expired())
    .unwrap_or(false);
```

Unless the session cookie is set and has been encrypted using the correct algorithm and secret key, calling [`session`](https://docs.rs/rwf/latest/rwf/http/request/struct.Request.html#method.session) will return `None`.

#### Expired sessions
If the session is expired, it's advisable not to trust its point of origin. While the contents are guaranteed to be accurate, the browser sending the data has not been validated in several weeks (4 weeks, by default).

The session can be used to privately store custom user-specific data. This allows your web apps to persist sensitive data on the client without using `localStorage` and JavaScript encryption.

### Session authentication

Rwf can ensure all requests have valid and current (not expired) sessions. To enable this feature, enable the [`SessionAuth`](https://docs.rs/rwf/latest/rwf/controller/auth/struct.SessionAuth.html) [authentication](../authentication) on your controllers.

## Store data in session

Rwf sessions allow you to store arbitrary JSON-encoded data. Since browsers place limits on cookie sizes, this data should be relatively small. To store some data in the session, you can set it on the [response](../response):

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

Sessions are automatically renewed on each request. Expired sessions are renewed as well, unless session [authentication](../authentication) is enabled.

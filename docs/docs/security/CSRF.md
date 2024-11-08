# CSRF protection

Cross-site request forgery[^1] (or CSRF) is a type of attack which uses your website's forms to trick the user into submitting data to your application from somewhere else. Rwf comes with [middleware](../controllers/middleware.md) to protect your application against such attacks.

## Enable CSRF protection

CSRF protection is enabled by default. When users make `POST`, `PUT`, and `PATCH` requests to your app, Rwf will check for the presence of a CSRF token. If the token is not there, or has expired, the request will be blocked and `HTTP 400 - Bad Request` response will be returned.

## Passing the token

The CSRF token can be passed using one of two methods:

- `X-CSRF-Token` HTTP header
- `<input name="rwf_csrf_token" type="hidden">` inside a form

If you're submitting a form, you can add the `rwf_csrf_token` input automatically:

```html
<form method="post" action="/login">
    <%= rwf_token() %>
</form>
```

If you're making AJAX requests (using `fetch`, for example), you can pass the token via the header. If you're using Stimulus (which comes standard with Rwf), you can pass the token via a data attribute to the Stimulus controller:

=== "HTML"
    ```html
    <div
      data-controller="login"
      data-csrf-token="<%= rwf_token_raw() %>"
    >
      <!-- ... -->
    </div>
    ```
=== "JavaScript"
    ```javascript
    import { Controller } from "hotwired/stimulus"

    export default class LoginController extends Controller {

      // Send request with CSRF token included.
      sendRequest() {
        const csrfToken = this.element.dataset.csrfToken;

        fetch("/login", {
          headers: {
            "X-CSRF-Token": csrfToken,
          }
        })
      }

    }
    ```

## Disable CSRF protection

If you want to disable CSRF protection, you can do so globally by toggling the `csrf_protection` [configuration option](../configuration.md) to `false`, or on the controller level by implementing the `fn skip_csrf(&self)` method:

```rust
use rwf::prelude::*;

#[derive(Default)]
struct IndexController;

impl Controller for IndexController {
    /// Disable CSRF protection for this controller.
    fn skip_csrf(&self) -> bool {
        true
    }

    /* ... */
}
```

### REST

If you're using JavaScript frameworks like React or Vue for your frontend, it's common to disable CSRF protection on your [REST](../controllers/REST/index.md) controllers. To do so, you can add the `#[skip_csrf]` attribute to your `ModelController`, for example:

```rust
#[derive(macros::ModelController)]
#[skip_csrf]
struct Users;
```

You can always disable CSRF globally via [configuration](#disable-csrf-protection) and enable it only on the controllers that serve HTML forms.

## Token validity

The CSRF token is valid for the same duration as Rwf [sessions](../controllers/sessions.md). By default, this is set to 4 weeks. A new token is generated every time your users load a page which contains a token generated with the built-in template functions.

## WSGI / Rack controllers

Rwf CSRF protection is disabled for [Python](../migrating-from-python.md) and [Rails](../migrating-from-rails.md) applications. It's expected that Django/Flask/Rails applications will use their own CSRF protection middleware.

[^1]: [https://en.wikipedia.org/wiki/Cross-site_request_forgery](https://en.wikipedia.org/wiki/Cross-site_request_forgery)

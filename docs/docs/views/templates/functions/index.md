# Functions overview

Templates provide a number of functions that manipulate constants and variables. Each data type has its own set of functions, which you can call using the dot (`.`) notation, for example:

=== "Template"
    ```erb
    <%= "lowercase".upper %>
    ```
=== "Output"
    ```
    LOWERCASE
    ```

## Functions

- [String functions](string.md)
- [Integer functions](integer.md)
- [Float functions](float.md)
- [Hash functions](hash.md)
- [List functions](list.md)

## General helpers

These functions can be called on any value, irrespective of data type.

### `null`

Return true if the value is null, false if not.

```erb
<h1>
  <% if title.null %>
    Unnamed
  <% else %>
    <%= title %>
  <% end %>
</h1>
```

Aliases:

- `nil`
- `blank`

### `numeric`

Return true if the value is a number, i.e. integer or float. Return false if not.

```erb
<% if value.numeric %>
  <input type="number">
<% else %>
  <input type="text">
<% end %>
```

### `integer`

Return true if the value is an integer, false otherwise.

```erb
<% 5.integer == true %>
```

### `float`

Return true if the value is an integer, false otherwise.

```erb
<% 5.float == false %>
```

## Global helpers

Global functions are standalone and are not called on a value. They are used to generate some useful code in the template.

### `rwf_head`

Inserts JavaScript into template that makes Rwf work smoothly. Currently this function downloads and initializes Hotwired Turbo and Stimulus libraries. As the name of the function suggests, it's best used inside the `<head>` element, for example:

```html
<!doctype html>
<html>
  <head>
    <%- rwf_head() %>
  </head>
  <body>
    <!-- ... -->
```

### `rwf_turbo_stream`

Inserts JavaScript code which will create and initialize a [Turbo Stream](../../turbo/streams.md) WebSocket connection. Use this function inside the `<body>` element[^1]:

```html
<!doctype html>
<html>
  <head>
    <%- rwf_head() %>
  </head>
  <body>
    <%- rwf_turbo_stream("/turbo-stream") %>
    <!-- ... -->
```

[^1]: [https://turbo.hotwired.dev/handbook/streams](https://turbo.hotwired.dev/handbook/streams)


### `render`

Renders a template directly inside the current template. Can be used for rendering [partials](../partials.md). `<%%` is a special template code tag which is an alias for `render`.

```html
<div>
  <%- render("templates/profile.html") %>
</div>

<!-- The same as: -->

<div>
  <%% "templates/profile.html" %>
</div>
```

### `csrf_token`

Renders an input field with a valid [CSRF](../../../security/CSRF.md) token.

```html
<form action="/login" method="post">
    <%= csrf_token() %>
</form>
```


### `csrf_token_raw`

Renders a valid [CSRF](../../../security/CSRF.md) token as a raw HTML string. It can then be passed to JavaScript via a `data-` attribute or a global variable:

```html
<div data-csrf-token="<%= csrf_token_raw() %>"
</div>
```

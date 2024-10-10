
# Dynamic templates

Rum has its own template language, heavily inspired (if not shamelessly copied) from Rails' ERB.

## Quick example

If you've used Rails before, you'll find this syntax familiar:

```erb
<p><%= text %></p>

<ul>
<% for item in list %>
    <li><%= item.upcase %><li>
<% end %>
<ul>

<script>
<%- no_user_inputs_allowed_code %>
</script>

<% if value == "on" %>
    <p>Now you see me</p>
<% else %>
    <p>Now you don't</p>
<% end %>
```

## Operations

Rum's templates syntax is very small and simple:

| Operation | Description |
|----------|-------------|
| `<%` | Code block start. |
| `%>` | Code block end. |
| `<%=` | Print the following expression value (don't forget to close the code block). |
| `<%-` | Print expression without escaping "dangerous" HTML characters. |
| `<% if expression %>` | If block which evaluates the expression for truthiness. |
| `<% elsif expression %>`| Else if block, works just like the if block. |
| `<% else %>` | Else block. |
| `<% for item in list %>` | For loop. |
| `<% end %>` | Indicates the end of an if statement or for loop. |
| `+`, `-`, `*`, `/`, `==`, `%` | Addition, subtraction, multiplication, division, equality, modulo. |

## Rendering templates

Templates can be rendered directly from a Rust string:

```rust
#[derive(rum::macros::Context)]
struct Index {
    first_name: String,
    user_id: i64,
}

let template = Template::from_str("<p>Ahoy there, <%= first_name %>! (id: <%= user_id %></p>")?;
let context = Index { first_name: "Josh".into(), user_id: 1 };

let result = template.render(context.try_into()?)?;

assert_eq!(result, "<p>Ahoy there, Josh! (id: 1)</p>");
```

Templates can be placed in files anywhere the Rust program can access them:

```rust
let template = Template::load("templates/index.html").await?;
let result = template.render(context.try_into()?)?;
```

`templates/index.html` is a path relative to current wording directory (`$PWD`).

Templates don't have to be HTML, and can be used to render any kind of files, e.g. plain text, CSS, JavaScript, etc.

## Passing values to templates

Rum's templates support many data types, e.g. strings, integers, lists, hashes, and even models. For example, a list of users can be passed directly into a template:

```rust
let users = User::all()
    .fetch_all(&mut conn)
    .await?;

let template = Template::from_str(
"<ul>
    <% for user in users %>
        <li><%= user.email %></li>
    <% end %>
</ul>")?;

#[derive(rum::macros::Context)]
struct Context {
    users: Vec<User>,
}

let context = Context { users };

let rendered = template.render(&context.try_into()?)?;
```

## Data types

Multiple Rust data types are supported out of the box and each data type comes with its own operations.

| Template data type | Rust data type |
|-----------|---------|
| Integer | `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64` |
| Float | `f32`, `f64` |
| String | `String`, `&str` |
| List | | `Vec` with any Rust data type, including the ORM's models |
| Hash | `HashMap` of any Rust data type |

### Operations

Each template data type supports its own operations.

#### Number

| Operation | Description | Example |
|-----------|-------------|---------|
| `abs` | Get the absolute value (non-negative) | `<%= 25.abs %>` |
| `to_string`, `to_s` | Convert the number to a string | `<% if 25.to_s == "25" %>` |
| `to_f` | `to_float` | Convert the number to a floating point number | `<% if 25.to_f == 25.0 %>` |
| `times` | Create a list of numbers enumerated from 0 to the number | `<% for i in 25.times %>` |

#### Float

| Operation | Description | Example |
| `abs` | Get the absolute value (non-negative) | `<%= -25.0.abs %>` |
| `ceil` | Ceil the floating point to the nearest integer | `<% if 25.5.ceil == 26 %>` |
| `floor` | Floor the floating point to the nearest integer | `<% if 25.5.ceil == 25 %>` |
| `round` | Round the floating point to the nearest integer | `<% if 25.5.ceil == 26 %>` |
| `to_string`, `to_s` | Convert the floating point to a string representation. | `<%= 25.5.to_s %>` |

## Caching templates

Reading templates from disk is usually quick, but compiling them can take some time. In development, they are compiled every time they are fetched, which allows to iterate on their contents quickly, while in production they are cached in memory for performance.

The caching behavior is controlled via configuration and requires no code modifications:

```toml
[general]
cache_templates = true
```

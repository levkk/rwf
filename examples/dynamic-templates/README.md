
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
let template = Template::cached("templates/index.html").await?;
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

## Caching templates

Reading templates from disk is usually quick, but compiling them can take some time. In development, they are compiled every time they are fetched, which allows to iterate on their contents quickly, while in production they are cached in memory for performance.

The caching behavior is controlled via configuration and requires no code modifications:

```toml
[general]
cache_templates = true
```

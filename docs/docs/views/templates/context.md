# Template context

Template context is a collection of variables that are passed to a template during rendering. The variables in the template
are replaced with the values in the context, producing the final rendering of the template.

Using different contexts for the same template allows for templates to be re-used in different parts of the application,
without having to write the same HTML/CSS/JavaScript multiple times.

## Create a context

A context is a key/value mapping of variable names to variable values. Rwf allows it to be created in many different ways, depending on the situation, but the simplest is to use the `context!` macro:

```rust
let ctx = context!(
    "title" => "Empire strikes back",
    "r2d2_password" => vec![
        1_i64, 1, 2, 3, 5, 8,
    ],
);
```

The values passed to the `context!` macro are automatically converted from Rust into the right template data type.

### Type-safe context

If you want to re-use the same context in multiple places or you want to ensure the right types are passed in at creation, you can define a struct and convert it into a context using the `macros::Context` derive:

```rust
#[derive(macros::Context)]
struct Variables {
    title: String,
    r2d2_password: Vec<i64>,
}
```

You can instantiate this context now like a regular struct and pass it to a template, for example:

```rust
let ctx = Variables {
    title: "The last Jedi".to_string(),
    r2d2_password: vec![1, 1, 2, 2, 3, 3],
};
let rendered = template.render(&ctx)?;
```

## Learn more

- [For loops](for-loops.md)
- [If statements](if-statements.md)
- [Templates in controllers](templates-in-controllers.md)

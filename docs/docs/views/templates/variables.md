# Variables

Template variables are used to substitute unique information into a reusable template. Rwf supports variables of different kinds, like strings, numbers, lists, and hashes. Complex variables like hashes can be iterated through using [for loops](../for-loops).

## Using variables

Using variables in your templates is typically done by "printing" them, or outputting them, into the template text, by placing them between `<%=` and `%>` tags:

```erb
<%= variable %>
```

The `<%=` tag indicates what follows is an [expression](../nomenclature), which should be evaluated and converted to text for displaying purposes.

The `%>` tag is not specific to printing variables, and indicates the end of a code block inside a template. What follows that tag is just regular text which has no special meaning.

## Defining variables

A variable is defined when a template is rendered. Using one of many possible ways to define a [context](../context), the variable is given a value at runtime:

=== "Rust"
    ```rust
    let template = Template::from_str("<%= variable %>")?;
    let string = template.render([
        ("variable", "I love pancakes for dinner."),
    ])?;
    println!("{}", string);
    ```
=== "Output"
    ```
    I love pancakes for dinner.
    ```

### Missing variables

It's not uncommon to forget to define variables, especially if a template is large, or used in multiple places in the app where some variables don't have a known value.

If an undefined variable is used in a template, Rwf will throw a runtime error. This is good for debugging issues when variables are unintentionally forgotten by the developer. However, if the variable is not always available, you can check if it's defined first:

```erb
<% if variable %>
  <p><%= variable %></p>
<% end %>
```

Due to the nature of [if statements](../if-statements), if the variable is defined and evaluates to a "falsy" value, e.g. `0`, `""` (empty string), `null`, etc., the if statement will not be executed either. This is helpful for handling many similar cases without having to write complex statements.

## Supported data types

Rwf variables support most Rust data types. The conversion between Rust and the template language happens automatically.

### Number

Rwf supports two kinds of numbers: integers and floating points.

An integer is any whole number, negative or positive (including zero). Rust has many integer types, e.g. `i8`, `i32`, `u64`, etc., but the template language converts all of them to an 64-bit singed integer:

=== "Template"
    ```erb
    <%= 500 %>
    ```
=== "Output"
    ```
    500
    ```

Rust's `f32` and `f64` are converted to 64-bit double precision floating point. Operations between integers and floating points are supported, the final result being a float:

=== "Template"
    ```erb
    <%= 500 + 1.5 %>
    ```
=== "Output"
    ```
    501.5
    ```

Numbers can be [converted](../functions) to strings, floored, ceiled and rounded, for example:

=== "Template"
    ```erb
    <%= 123.45.round.to_s %>
    ```
=== "Output"
    ```
    123
    ```

### Strings

Using strings in templates can be used in two ways: with the `<%=` (print) operator, which outputs the string, escaping any dangerous HTML characters, e.g. `<` becomes `&lt;`, or using `<%-` which performs no conversions and prints the string as-is.

#### String security

Escaping HTML characters is a good idea in cases where your users are the ones supplying the value of the string. This prevents script injection attacks, e.g. users placing malicious code on your website.

Unless you're sure about the provenance of a string, use `<%=` to output it in templates.

### Boolean

Boolean variables can either be `true` or `false`. They map directly to Rust's `bool` data type.

### Lists

Lists are arrays of other template variables, including other lists, strings, numbers, and hashes. In templates, lists can be defined by using square brackets, and iterated on using [for loops](../for-loops), for example:

=== "Template"
    ```erb
    <% for item in [1, 2, "three"] %>
    <%= item %>
    <% end %>
    ```
=== "Output"
    ```
    1
    2
    three
    ```

Rwf lists are flexible and can contain multiple data types. This separates them from Rust's `Vec` and slices which can only hold one kind of data.

You can also access a specific element in a list by indexing into it with the dot(`.`) notation:

```erb
<%= list.1 %>
```

Lists are 0-indexed, so the above example accesses the second element in the list.

### Hashes

Hashes, also known as dicts or hash tables, are a key/value storage data type. Unlike a list, it contains a mapping between a value (the key), and a value. Values can be accessed by knowing a key, or by iterating through the entire hash with a [for loop](../for-loops):

```erb
<p><%= user.name %></p>
<p><%= user.email %></p>
```

Rwf hashes use the dot (`.`) notation to access values in a hash. In this example, the `user` is a hash, and `name` and `email` are keys.

## Truthy vs. falsy

Variables are often used in [if statements](../if-statements) to decide whether to execute some code or not. To make the template language less verbose, variables can be evaluated for truthiness without calling explicit functions depending on their data type.

The following variables and data types evaluate to false:

| Data type | Value |
|-----------|----------|
| Integer | `0` |
| Float | `0.0` |
| Boolean | `false` |
| String | `""` (empty) |
| List | `[]` (empty) |
| Hash | `{}` (empty) |

All other variables evaluate to true.

## Learn more

- [If statements](../if-statements)
- [For loops](../for-loops)
- [Functions](../functions)

# Templates overview

Dynamic templates are a mix of HTML and a programming language which directs how the HTML is displayed. For example, if you have a profile page for your web app users, you would want each of your users to have a page unique to them. To achieve this, you would write only one template and substitute unique aspects of each using template variables, for example:

```erb
<div class="profile">
  <h2><%= username %></h2>
  <p><%= bio %></p>
</div>
```

The variables `username` and `bio` can be substituded for values unique to each of your users, for example:

=== "Rust"
    ```rust
    use rwf::prelude::*;

    let template = Template::from_str(r#"
    <div class="profile">
      <h2><%= username %></h2>
      <p><%= bio %></p>
    </div>
    "#)?;

    let html = template.render([
      ("username", "Alice"),
      ("bio", "I like turtles")
    ])?;

    println!("{}", html);
    ```
=== "Output"
    ```html
    <div class="profile">
      <h2>Alice</h2>
      <p>I like turtles</p>
    </div>
    ```

Templates help reuse HTML (and CSS, JavaScript) just like regular functions and structs help
reuse code.

## Learn more

- [Variables](variables)
- [For loops](for-loops)
- [If statements](if-statements)

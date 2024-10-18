# If statements

If statements allow you to control the flow of templates, conditionally displaying some elements while hiding others. For example, if a [variable](../variables) is "falsy", you can hide entire sections of your website:

```erb
<% if logged_in %>
  <!-- profile page -->
<% else %>
  <!-- login page -->
<% end %>
```

If statements start with `if` and must always finish with `end`.

## Expressions

If statements support evaluating large expressions for truthiness, for example:

```erb
<% if var.lower + "_" + bar.upper == "lo_HI" %>
  <!-- do something -->
<% end %>
```

While it's advisable to write simple if statements and delegate complex logic to views where the Rust compiler can be more helpful, Rwf template language is almost [Turing-complete](https://en.wikipedia.org/wiki/Turing_completeness) and can be used to write arbitrarily complex templates.

### Operator precedence

Templates respect operator precedence, e.g., multiplication is performed before addition, unless parentheses are specified (which are also supported).

## Else If

If statements support else if blocks (written as `elsif`), evaluating multiple expressions and executing the first one which evaluates to true:

```erb
<% if one %>
  <!-- one -->
<% elsif two %>
  <!-- two -->
<% elsif three %>
  <!-- three -->
<% else %>
  <!-- I guess it's four? --->
<% end %>
```

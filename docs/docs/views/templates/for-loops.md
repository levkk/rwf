# For loops

Rwf templates have only one kind of for loop: for each. This allows writing more reliable templates, and to avoid common bugs like infinite loops, which will stall a web app in production.

A for loop can iterate over a list of values, for example:

```erb
<ul>
<% for value in list %>
  <li><%= value %></li>
<% end %>
</ul>
```

Template lists, unlike Rust's `Vec`, can hold [variables](../variables) of different data types, and are dynamically evaluted at runtime:

=== "Template"
    ```erb
    <% for value in ["one", 2 * 5, 3/1.5] %>
    <%= value %>
    <% end %>
    ```
=== "Output"
    ```
    one
    10
    2.0
    ```

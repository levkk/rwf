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

Template lists, unlike Rust's `Vec`, can hold [variables](../variables) of different data types, and are dynamically evaluated at runtime:

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

##  Do _n_ times

If you need to execute some code multiple times, templates come with a handy `times` function:

=== "Template"
    ```erb
    <% for n in 5.times %>
      <li><%= n %>.</li>
    <% end %>
    ```
=== "Output"
    ```
    <li>1.</li>
    <li>2.</li>
    <li>3.</li>
    <li>4.</li>
    <li>5.</li>
    ```

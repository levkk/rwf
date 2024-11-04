# Hash functions

### `keys`

Converts the hash to a list of keys stored in the hash.

=== "Template"
    ```erb
    <% for fruit in fruits.keys %>
        <%= fruit %>
    <% end %>
    ```
=== "Context"
    ```rust
    context!(
        "fruits" => HashMap::from([
            ("apples", "red"),
            ("bananas", "yellow")
        ])
    );
    ```
=== "Output"
    ```
    apples
    bananas
    ```

### `values`

Converts the hash to a list of values stored in the hash.

=== "Template"
    ```erb
    <% for color in fruits.values %>
        <%= color %>
    <% end %>
    ```
=== "Context"
    ```rust
    context!(
        "fruits" => HashMap::from([
            ("apples", "red"),
            ("bananas", "yellow")
        ])
    );
    ```
=== "Output"
    ```
    red
    yellow
    ```

### `iter`

Converts the hash to a list of key & value tuples that are stored in the hash. When used inside a for loop, calling `iter` is optional.

=== "Template"
    ```erb
    <% for tuple in fruits %>
        <%= tuple.0 %> are <%= tuple.1 %>
    <% end %>
    ```
=== "Context"
    ```rust
    context!(
        "fruits" => HashMap::from([
            ("apples", "red"),
            ("bananas", "yellow")
        ])
    );
    ```
=== "Output"
    ```
    apples are red
    bananas are yellow
    ```


### `empty`

Returns true if the hash is empty (length 0). `blank` and `is_empty` are aliases for `empty`.

=== "Template"
    ```erb
    <%= fruits.empty %>
    ```
=== "Context"
    ```rust
    context!(
        "fruits" => HashMap::new(),
    );
    ```
=== "Output"
    ```
    true
    ```


### `len`

Returns the length of the hash, i.e. the number of elements stored in the hash.

=== "Template"
    ```erb
    <%= fruits.len %>
    ```
=== "Context"
    ```rust
    context!(
        "fruits" => HashMap::new(),
    );
    ```
=== "Output"
    ```
    0
    ```


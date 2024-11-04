# List functions

### `enumerate`

Converts the list to a new list of tuples. Each tuple contains the element's position in the original list and its value.

=== "Template"
    ```erb
    <% for tuple in [1, 2, 3].enumerate %>
        <%= tuple.0 %>. <%= tuple.1 %>
    <% end %>
    ```
=== "Output"
    ```
    0. 1
    1. 2
    2. 3
    ```

### `reverse`

Reverses the order of elements in the list. `rev` is an alias for `reverse`.

=== "Template"
    ```erb
    <% for value in [1, 2, 3].reverse %>
        <%= value %>
    <% end %>
    ```
=== "Output"
    ```
    3
    2
    1
    ```

### `len`

Returns the length of the list.

=== "Template"
    ```erb
    <%= [1, 2, 3].len %>
    ```
=== "Output"
    ```
    3
    ```

### `empty`

Returns true if the list is empty (length 0).

=== "Template"
    ```erb
    <%= [1, 2, 3].empty %>
    ```
=== "Output"
    ```
    false
    ```

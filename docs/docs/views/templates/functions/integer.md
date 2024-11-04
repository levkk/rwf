
# Integer functions

### `abs`

Returns the absolute (non-negative) value of the integer.

=== "Template"
    ```erb
    <%= -5.abs %>
    ```
=== "Output"
    ```
    5
    ```

### `to_string`

Converts the integer to a string. `to_s` is an alias for `to_string`.

=== "Template"
    ```erb
    <%= 5.to_string + " times" %>
    ```
=== "Output"
    ```
    5 times
    ```

### `to_float`

Converts the integer to a floating point number. `to_f` is an alias for `to_float`.

=== "Template"
    ```erb
    <%= 5.to_float * 2.5 %>
    ```
=== "Output"
    ```
    12.5
    ```


### `times`

Creates a list of integers, starting at 0 and ending with the integer. This function is commonly used to run a for loop, for example:


=== "Template"
    ```erb
    <% for i in 3.times %>
        <%= i %>.
    <% end %>
    ```
=== "Output"
    ```
    1.
    2.
    3.
    ```

### `clamp_zero`

Clamp the integer to 0, i.e. negative values become 0.

=== "Template"
    ```erb
    <%= -25.clamp_zero %>
    ```
=== "Output"
    ```
    0
    ```

### `clamp_one`

Clamp the integer to 1, i.e. all values less than 1 become 1.

=== "Template"
    ```erb
    <%= 0.clamp_one %>
    ```
=== "Output"
    ```
    1
    ```

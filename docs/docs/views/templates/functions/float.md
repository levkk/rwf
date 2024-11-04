# Float functions

### `abs`

Returns the absolute (non-negative) value of the floating point.

=== "Template"
    ```erb
    <%= -5.0.abs %>
    ```
=== "Output"
    ```
    5.0
    ```

### `to_string`

Converts the floating point to a string. `to_s` is an alias for `to_string`.

=== "Template"
    ```erb
    <%= 5.2.to_string + " times" %>
    ```
=== "Output"
    ```
    5.2 times
    ```

### `to_integer`

Converts the floating point to an integer, rounding it. `to_i` is an alias for `to_integer`.

=== "Template"
    ```erb
    <%= 5.2.to_integer * 5 %>
    ```
=== "Output"
    ```
    25
    ```

### `round`

Rounds the floating point to the nearest whole value.

=== "Template"
    ```erb
    <%= 5.6.round %>
    ```
=== "Output"
    ```
    6.0
    ```

### `ceil`

Rounds the float to the upper whole value.

=== "Template"
    ```erb
    <%= 5.2.ceil %>
    ```
=== "Output"
    ```
    6.0
    ```

### `floor`

Rounds the float to the lower whole value.

=== "Template"
    ```erb
    <%= 5.9.ceil %>
    ```
=== "Output"
    ```
    5.0
    ```

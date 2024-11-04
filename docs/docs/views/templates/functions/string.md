
# String functions

### `to_uppercase`

Converts the string to uppercase lettering. `upper` is an alias for `to_uppercase`.

=== "Template"
    ```erb
    <%= "name".to_uppercase %>
    ```
=== "Output"
    ```
    NAME
    ```

### `to_lowercase`

Converts the string to lowercase lettering. `lower` is an alias for `to_lowercase`.

=== "Template"
    ```erb
    <%= "NAME".to_lowercase %>
    ```
=== "Output"
    ```
    name
    ```

### `trim`

Removes leading and trailing spaces and new line characters from the string.

=== "Template"
    ```erb
    <%= " value ".trim + " ,name ".trim %>
    ```
=== "Output"
    ```
    value,name
    ```


### `capitalize`

Capitalizes the first letter of the string.

=== "Template"
    ```erb
    <%= "john".capitalize %>
    ```
=== "Output"
    ```
    John
    ```


### `underscore`

Converts the string to "snake_case" formatting. `to_snake_case` is an alias for `underscore`.

=== "Template"
    ```erb
    <%= "ClassName".underscore %>
    ```
=== "Output"
    ```
    class_name
    ```

### `camelize`

Converts the string to "CamelCase" formatting.

=== "Template"
    ```erb
    <%= "class_name".camelize %>
    ```
=== "Output"
    ```
    ClassName
    ```

### `empty`

Returns true if the string is empty (length 0). `blank` and `is_empty` are aliases for `empty`.

=== "Template"
    ```erb
    <%= "".empty %>
    ```
=== "Output"
    ```
    true
    ```

### `len`

Returns the length of the string.

=== "Template"
    ```erb
    <%= "hello".len %>
    ```
=== "Output"
    ```
    5
    ```

### `urldecode`

Replaces percent-encoding in the string with its ASCII character equivalents. Commonly used to send characters with special meaning inside URLs.

=== "Template"
    ```erb
    <%= "hello%3Dworld".urldecode %>
    ```
=== "Output"
    ```
    hello=world
    ```

### `urlencode`

Opposite of `urldecode`. Replaces ASCII characters with special meaning in URLs with percent-encoded strings.

=== "Template"
    ```erb
    <%= "hello=world".urlencode %>
    ```
=== "Output"
    ```
    hello%3Dworld
    ```


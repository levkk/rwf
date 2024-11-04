# Functions

Templates provide a number of functions that manipulate variables. Each variable data type has its own set of functions, which you can call using the dot (`.`) notation, for example:

=== "Template"
    ```erb
    <%= "lowercase".upper %>
    ```
=== "Output"
    ```
    LOWERCASE
    ```

## Any value functions

Functions that can be called on any value, irrespective of type.

| Function | Description |
|----------|-------------|
| `null` | Return true if the value is null, false otherwise. |
| `nil` | Alias for `null` |
| `numeric` | True if the value is a number, e.g. integer or float. |
| `integer` | True if the value is an integer. |
| `float` | True if the value is a float. |

## Integer functions

| Function | Description |
|----------|-------------|
| `abs` | Get the absolute (non-negative) value. |
| `to_string` | Convert an integer to a string. |
| `to_s` | Alias for `to_string`. |
| `to_float` | Convert integer to float. |
| `to_f` | Alias for `to_float`. |
| `times` | Create a list of integers, starting at 0 and ending with the integer. |
| `clamp_zero` | Clamp integer to 0, i.e. all negative values become 0. |
| `clamp_one` | Clamp integer to 1, i.e. all values less than 1 become 1. |

### Examples

Getting the absolute value of an integer:

=== "Template"
    ```erb
    <% if -5.abs == 5 %>
    <h1>True<h1>
    <% end %>
    ```
=== "Output"
    ```
    <h1>True</h1>
    ```

Converting an integer to a string:

=== "Template"
    ```erb
    <% if 25.to_s == "25" %>
    <h1>True<h1>
    <% end %>
    ```
=== "Output"
    ```
    <h1>True</h1>
    ```

## Float functions

| Function | Description |
|----------|-------------|
| `abs` | Get the absolute (non-negative) value. |
| `to_string` | Convert an integer to a string. |
| `to_s` | Alias for `to_string`. |
| `to_integer` | Convert float to integer, rounding it. |
| `to_i` | Alias for `to_integer`. |
| `round` | Round the float to the nearest whole value. |
| `ceil` | Round the float to the upper whole value. |
| `floor` | Round the float to the lower whole value. |

### Examples

Comparing a float and an integer:

=== "Template"
    ```erb
    <% if 25 == 25.4.to_i %>
    <h1>True<h1>
    <% end %>
    ```
=== "Output"
    ```
    <h1>True</h1>
    ```

## String functions

| Function | Description |
|----------|-------------|
| `to_uppercase` | Convert string to uppercase lettering. |
| `uppper` | Alias for `to_uppercase`. |
| `to_lowercase` | Convert string to lowercase lettering. |
| `lower` | Alias for `to_uppercase`. |
| `trim` | Remove leading and trailing spaces and new lines. |
| `capitalize` | Capitalize the first letter of a string. |
| `underscore` | Convert to snake_case. |
| `to_snake_case` | Alias for `underscore`. |
| `camelize` | Convert to PascalCase. |
| `empty` | True if string is empty. |
| `blank` | Alias for `empty`. |
| `is_empty` | Alias for `empty`. |
| `len` | Return length of the string. |
| `urldecode` | Convert percent-encoding to ASCII. |
| `urlencode` | Opposite of `urldecode`. |
| `capitalize` | Make the first letter of the string uppercase. |

### Examples

Trim a string with extra leading and trailing spaces:

=== "Template"
    ```erb
    <p><%= "  messy string  " %></p>
    ```
=== "Output"
    ```
    <p>messsy string</p>
    ```

## List functions

| Function | Description |
|----------|-------------|
| `enumerate` | Convert the list to a list of element position and element tuples. |
| `reverse` | Convert the list to a new list of elements positioned from end to beginning. |
| `rev` | Alias for `reverse`. |
| `len` | Get the list length. |
| `empty` | `true` if empty, `false` otherwise |

### Examples

Enumerate a list:

=== "Template"
    ```erb
    <% for tuple in ["one", "two"].enumerate %>
    <li><%= tuple.0 %> &dash; <%= tuple.1 %>
    <% end %>
    ```
=== "Output"
    ```
    <li>0 - one <li>
    <li>1 - two</li>
    ```

## Hashes

| Function | Description |
|----------|-------------|
| `keys` | Create a list of hash keys. |
| `values` | Create a list of hash values. |
| `iter` | Create a list of tuples, mapping keys to values. Used for iteration over a hash. |
| `len` | Get the hash length (how many keys are stored in it). |
| `empty` | `true` if empty, `false` otherwise |
| `blank` | Alias for `empty`. |
| `is_empty` | Alias for `empty`. |

## Global functions

Global functions are functions that can be used without a value, anywhere in the template.

| Function | Description |
|----------|-------------|
| `rwf_head` | Inject HTML and JavaScript that makes Rwf "just work". Use in the `<head>` element of a template. |
| `rwf_turbo_stream` | Inject code that will create a `<turbo-stream-source>` element pointing to the right endpoint. |
| `render` | Render a partial template. `<%%` is an alias for this function. |

=== "Head"
    ```erb
    <html>
      <head>
        <%- rwf_head() %>
        <title>Home page</title>
      </head>
    ```

=== "Turbo Stream"
    ```erb
    <html>
      <body>
        <%- rwf_turbo_stream("/turbo-stream-endpoint") %>
    ```

=== "Render"
    ```erb
    <html>
      <body>
        <%- render("templates/index.html") %>
    ```

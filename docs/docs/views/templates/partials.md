# Partials

Partials are templates that can be rendered inside other templates to facilitate reuse of frontend code. For example, if you have a navigation menu in your app, you would want it to look the same on all pages,
but you don't want to implement the same menu many times. To achieve this, you can write a partial and insert it in all templates where it's needed.

## Writing parials

A partial is just another template. It has to be stored on disk, in a directory reachable from your application, for example `templates/partials`.

Using the navigation menu as an example, we can define a partial in `templates/partials/nav.html`, as follows:

```html
<nav>
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/profile">Profile</a></li>
  </ul>
</nav>
```

Rendering partials can be done in any template, using the special `<%%` tag, for example:

=== "Template"
    ```html
    <html>
      <head>
      <!-- ... -->
      </head>
      <body>
        <%% "templates/partials/nav.html" %>
      </body>
    </html>
    ```
=== "Output"
    ```html
    <html>
      <head>
      <!-- ... -->
      </head>
      <body>
        <nav>
          <ul>
            <li><a href="/">Home</a></li>
            <li><a href="/profile">Profile</a></li>
          </ul>
        </nav>
      </body>
    </html>
    ```

## Using variables

Partials can use variables like a regular template. When rendered, they inherit the variables and scope of the template they are used in, for example:

=== "Partial"
    ```erb
    <h1><%= user.name %></h1>
    <p><%= user.bio %></p>
    ```
=== "Template"
    ```erb
    <% for user in users %>
      <%% "templates/partials/user.html" %>
    <% end %>
    ```

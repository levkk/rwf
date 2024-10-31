# Turbo basics

[Hotwired Turbo](https://turbo.hotwired.dev/) is a JavaScript library that can intercept HTTP requests to your backend and perform  updates to the frontend without reloading the browser page. The backend produces HTML, generated with [dynamic templates](../templates/), and Turbo updates only the sections of the page that changed. This simulates the behavior of [Single-page applications](https://en.wikipedia.org/wiki/Single-page_application) (like the ones written with React or Vue) without using JavaScript on the frontend.

## Enabling Turbo

If you're building pages using Rwf's [dynamic templates](../templates/), you can enable Turbo by adding a declaration into the `<head>` element of your pages:

```html
<html>
  <head>
    <%- rwf_head %>
  </head>
  <!-- ... -->
```

Otherwise, you can always get Turbo from a CDN, like [Skypack](https://www.skypack.dev/view/@hotwired/turbo).

## Using Turbo

Once Turbo is loaded, all links and forms will use Turbo automatically. When visiting links or submitting forms, Turbo will intercept the request, send it on the browser's behalf, process the response and replace the contents of the page seamlessly.

## Learn more

- [Turbo Streams](streams.md)

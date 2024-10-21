# Building pages

Turbo can be used to update parts of the page, without having to render the entire page on every request. This is useful when you want to update sections of the page from any endpoint, without having to load several [partials](../../templates/partials) or performing redirects.

Partial updates uses Turbo Streams, a feature of Turbo that sends page updates via [forms](../../../controllers/forms) or [WebSockets](../../../controllers/websockets).

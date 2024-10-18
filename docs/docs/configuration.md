# Configuration

Rwf supports file-based and environment-based configuration. The list of configurable options are ever growing, and currently supported features are listed below.

## Enabling configuration

To configure Rwf, place a file called `rwf.toml` into the wording directory of your app. During development, this should be the root directory of your Cargo project. At startup,
Rwf will automatically load configuration settings from that file, as they are needed by the application.

## Available settings

The configuration file is using the [TOML language](https://toml.io/). If you're not familiar with TOML, it's pretty simple and expressive language commonly used in the world of Rust programming.

Rwf configuration file is split into multiple sections. The `[general]` section controls various options such as logging settings, and which secret key to use for [encryption](../encryption). The `[database]`
section configures database connection settings, like the database URL, connection pool size, and others.

### `[general]`

| Setting | Description | Example |
|---------|-------------|---------|
| `log_queries` | Toggles logging of all SQL queries executed by the [ORM](../models/). | `log_queries = true` |
| `secret_key` | Secret key, encoded using base64, used for [encryption](../encryption). | `secret_key = "..."` |

#### Secret key

The secret key is a base64-encoded string of randomly generated data. A valid secret key contains 256 bits of entropy and _must_ be generated using a [_secure_](https://en.wikipedia.org/wiki/Cryptographically_secure_pseudorandom_number_generator) random number generator.

### `[database]`

| Setting | Description | Example |
|---------|-------------|---------|

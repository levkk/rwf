# Configuration

Rwf supports file-based and environment-based configuration. The list of configurable options are ever growing, and currently supported features are listed below.

## Enabling configuration

To configure Rwf, place a file called `rwf.toml` into the working directory of your app. During development, this should be the root directory of your Cargo project. At startup,
Rwf will automatically load configuration settings from that file, as they are needed by the application.

## Available settings

The configuration file is using the [TOML language](https://toml.io/). If you're not familiar with TOML, it's pretty simple and expressive language commonly used in the world of Rust programming.

Rwf configuration file is split into multiple sections. The `[general]` section controls various options such as logging settings, and which secret key to use for [encryption](security/encryption.md). The `[database]`
section configures database connection settings, like the database URL, connection pool size, and others.

### `[general]`

| Setting | Description | Default |
|---------|-------------|---------|
| `log_queries` | Toggles logging of all SQL queries executed by the [ORM](models/index.md). | `false` |
| `secret_key` | Secret key, encoded using base64, used for [encryption](security/encryption.md). | Randomly generated |
| `cache_templates` | Toggle caching of [dynamic templates](views/templates/index.md). | `false` in debug, `true` in release |
| `csrf_protection` | Validate the [CSRF](security/CSRF.md) token is present on requests that mutate your application (POST, PUT, PATCH). | `true` |

#### Secret key

The secret key is a base64-encoded string of randomly generated data. A valid secret key contains 256 bits of entropy and _must_ be generated using a [_secure_](https://en.wikipedia.org/wiki/Cryptographically_secure_pseudorandom_number_generator) random number generator.

If you have Python installed on your system, you can generate a secret key for Rwf in just a few lines of code:

=== "Python"
    ```python
    import base64
    import secrets

    secret = base64.b64encode(secrets.token_bytes(int(256/8)))
    print(secret)
    ```
=== "Output"
    ```
    BJ3Og8l/Q8f+fLvQpb9CP7uUu/VG1/+CN2a1f/QyHWY=
    ```
    !!! warning
        Do not use this example key in production. Always generate a new one and keep it secret.

### `[database]`

| Setting | Description | Default |
|---------|-------------|---------|
| `name`  | Name of the database to connect to. | Same as the `$USER` shell variable. If not set, default is `postgres`. |
| `user`  | Name of the user to connect with to the database. | `$USER`, or `postgres` if not set. |
| `url` | Fully-qualified database connection string. | `postgresql://{user}/localhost:5432/{name}`, where `{user}` and `{name}` are `name` and `user` configuration values. |

#### `url`

The database URL was originally created by [The Twelve Factor App](https://12factor.net/) and uses the URL format for specifying database connections. It follows a standard format, as follows:

```
driver://user:password@host:port/database_name
```

For connecting to PostgreSQL, the `driver` is `postgresql` (or `postgres` is also acceptable).

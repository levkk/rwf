# Security

The ORM is a common place in a web app where malicious users attempt to inject bad data in order to extract information
they should not have access to or to damage the web app in some way, by deleting important data for example.

To protect against what we call SQL injection attacks[^1], the Rwf ORM takes multiple precautions.

## Prepared statements

Rwf uses prepared statements which separate the query text from user-specified values. The values themselves cannot be injected into the query,
so the most common type of SQL injection attack becomes hard to impossible to execute.

You'll note that all of our SQL examples use placeholders, values starting with the `$` sign, to indicate where values should go. The placeholders
are replaced by the database once both the query text and the values are received, so malicious values can never leak into the query language itself.

## Escaping user-supplied values

For other values like column names, Rwf escapes them in order to avoid modifying queries in unexpected ways.

=== "Rust"
    ```rust
    let oops = User::all()
      .filter("\";DROP TABLE users;", 5)
      .fetch(&mut conn)
      .await?;
    ```
=== "SQL"
    ```postgresql
    SELECT * FROM "users" WHERE """;DROP TABLE users;" = 5;
    ```
=== "Error"
    ```
    ERROR:  column "";DROP TABLE users;" does not exist
    ```

[^1]: A SQL injection attack is injecting custom SQL into a query, in order to extract data from database tables.

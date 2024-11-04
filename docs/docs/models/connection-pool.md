# Connection pool

Rwf manages database connections automatically. Since Rwf apps are multi-threaded and asynchronous, a typical deployment will require multiple connections to the database to support concurrent requests. The connection pool takes care of creating and closing connections, and providing them to to the app as needed.

## Get a connection

To execute queries with the ORM, you'll need to check out a connection from the pool. You can do so as follows from anywhere in the code:

```rust
let mut conn = Pool::connection().await?;
```

Once you have a connection, you can pass it to the ORM each time you need to execute a query:

```rust
let users = User::all()
    .fetch_all(&mut conn)
    .await?;
```

## Return connection to the pool

Returning the connection to the pool is done automatically when the `conn` variable goes out of scope. In Rust semantics, the `conn` variable is "dropped". For example, to checkout a connection for only one query, you can do so inside its own scope:

```rust
let users = {
    let mut conn = Pool::connection().await?;
    let users = User::all()
        .fetch_all(&mut conn)
        .await?
};
```

## Transactions

All queries are executed inside implicit transactions. If you need to execute multiple queries inside a single transaction, you need to start one explicitly:

```rust
let mut transaction = Pool::transaction().await?;
```

The transaction follows the same scope semantics as a pool connection. When it goes out scope,
the transaction is automatically rolled back and the connection is returned back to the pool. If you want to commit any changes you made inside the transaction, you need to call `commit` explicitly:

```rust
transaction.commit().await?;
```

Automatic rollbacks are a safety feature of Rwf connection management. In case an error happens in Rust mid-transaction, the changes are automatically reverted, preventing partial updates to the database.

Just like a connection, the transaction can be passed to any query generated with the ORM:

```rust
let user = User::find(15)
    .fetch_one(&mut transaction)
    .await?;
```

## Waiting for connections

When all available connections are checked out, the call to `Pool::connection()` will wait (and asynchronously block) until a connection is returned to the pool. If a connection is not returned in time, an timeout error will be returned, unblocking the request and allowing it to handle the situation gracefully.

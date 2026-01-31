# Cursor
Rwf has support for Cursors - with all advantages and disadvantages they come with.
Every qualified (not data modifying) query can be used to create a Cursor.


## Basics
There are two base types of cursors. `ModelCursor` and `SelectiveCursor` the former is used fetch rows which can be mapped to a struct implements the `Model` trait. 
The later ist used to fetch arbitrary rows.

## Example

Just create a `Query` and use the `Query::declare` method to create Declare Statement. 
This can be used to configure the Cursor and provides Methods to construct a cursor then.

```rust
use rwf::prelude;
use rwf::model::prelude::*;
#[derive(Clone, macros::Model, Serialize, Deserialize)]
struct User {
    id: Option<i64>,
    name: String,
    mail: String
}
#[tokio::main]
async fn main() -> Result<(), rwf::model::Error> {
    let mut cursor = User::all()
        .filter_not_ends_with("mail", ".tld") 
        .order_by(("id", "asc")) // Always Order the underlying query to make results reproducible 
        .declare("cursor_name") // Give the cursor a name to refer in FETCH Queries
        .asensitive() // Make Table changes like inserts available to the cursor
        .scroll() // Allow all kind of fetches. If not sepcified only forward directed fetches are allowed
        .hold() // Allow creation outside a transaction. 
        .create_tx_model_cursor(None) // Creat3 a cursor which fetches the Queries Model. Create a new transaction for the cursor. 
        .await?;
    let first_user = cursor.fetch_one(cursor.fetch_stmt().next()).await?;
    Ok(())
}
```
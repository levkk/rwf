# Complex Query
Rwf implements a wide range of postgres query features. 
So it is possible to use other queries defined in a WITH statement as well as COMBINED queries (UNION, EXCEPT, INTERSECT). 

## Example

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
    let query = User::all().except
    
    Ok(())
}
```
# Callbacks

RWF comes with its own callback infrastructure. This means Users are able to let the framework execute arbitrary functions whenever a Model is Created/Changed/Deleted,

## Create a Callback

To create a Callback, just define a struct (which could be used to transport data to the callback implementation) and implement the Callback Trait with the Associated Model

```rust
use rwf::prelude::*;
#[derive(Debug, Clone, Serialize, Deserialize, macros::Model)]
struct User {
    id: Option<i64>,
    mail: String,
    name: String,
}
struct UserCallback;
#[async_trait]
impl Callback<User> for UserCallback {
    async fn callback(self, data: User) -> User {
        // ToStuff
        data
    }
}
```
## Register a Callback

Once you have created the Callback, register it Somewhere in the Runtime (like the main function)

```rust
#[tokio::main]
async fn main() -> Result<(), rwf::http::Error> {
    /// Execute the `Callback` on every `User::create` or `User::save` (where the model not exists yet) - also known as Insert Query
    register_callback!(UserCallback, CallbackKind::Insert);
    /// Execute the `Callback` on every `User::save` (where the Model already exists) or `User::update_all` - also known as Update Query
    register_callback!(UserCallback, CallbackKind::Update);
    /// Execute the `Callback` on every `User::destroy` - also known as Delete Query
    register_callback!(UserCallback, CallbackKind::Delete);

    /// Do Stuff

    Ok(())
}
```

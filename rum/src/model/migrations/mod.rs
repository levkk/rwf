pub mod model;
pub use model::Migration;

pub async fn migrate() -> Result<Vec<Migration>, super::Error> {
    Migration::sync().await
}

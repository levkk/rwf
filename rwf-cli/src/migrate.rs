use rwf::model::migrations::{Direction, Migrations};

pub async fn migrate(version: Option<i64>) {
    let migrations = Migrations::sync().await.expect("failed to sync migrations");

    migrations
        .apply(Direction::Up, version)
        .await
        .expect("failed to apply migrations");
}

pub async fn revert(version: Option<i64>) {
    let migrations = Migrations::sync().await.expect("failed to sync migrations");

    migrations
        .apply(Direction::Down, version)
        .await
        .expect("failed to apply migrations");
}

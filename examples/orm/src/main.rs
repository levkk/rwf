// #![allow(dead_code)]
use rum::model::Migrations;
use rum::prelude::*;

mod models {
    use rum::model::prelude::*;
    use time::{Duration, OffsetDateTime};

    #[derive(Clone, rum::macros::Model, Debug)]
    #[has_many(Task)]
    pub struct User {
        pub id: Option<i64>, // id column is assigned by the database, new models don't have it until they are saved.
        pub email: String,
        pub created_at: OffsetDateTime,
        pub admin: bool,
        pub completed_tasks: i64,
    }

    impl User {
        pub async fn create_user(email: &str) -> Result<Self, Error> {
            let mut conn = Pool::connection().await?;

            let user = User::find_or_create_by(&[("email", email)])
                .fetch(&mut conn)
                .await?;

            Ok(user)
        }

        pub async fn add_task(&self, task_name: &str) -> Result<Task, Error> {
            let mut conn = Pool::connection().await?;

            let task = Task::create(&[
                ("name", task_name.to_value()), // Cast Rust value to `Value`.
                ("user_id", self.id.to_value()),
            ])
            .fetch(&mut conn)
            .await?;

            Ok(task)
        }

        pub async fn complete_all_tasks(&self) -> Result<Vec<Task>, Error> {
            let tasks = Pool::pool()
                .with_transaction(|mut transaction| async move {
                    // Exclusive lock on the user row, serializing updates to a row.
                    let _lock = self.tasks().lock().execute(&mut transaction).await?;

                    let tasks = self
                        .incomplete_tasks()
                        .update_all(&[("completed_at", OffsetDateTime::now_utc())])
                        .fetch_all(&mut transaction)
                        .await?;

                    let completed = self
                        .tasks()
                        .not("completed_at", Value::Null)
                        .count(&mut transaction)
                        .await?;

                    User::find(self.id)
                        .update_all(&[("completed_tasks", completed)])
                        .execute(&mut transaction)
                        .await?;

                    // Transaction has to be committed manually or it'll be rolled back.
                    transaction.commit().await?;

                    Ok(tasks)
                })
                .await?;

            Ok(tasks)
        }

        pub async fn make_admin(mut self) -> Result<Self, Error> {
            self.admin = true;
            let mut conn = Pool::connection().await?;
            Ok(self.save().fetch(&mut conn).await?)
        }

        pub async fn remove_admin(mut self) -> Result<Self, Error> {
            self.admin = false;
            Pool::pool()
                .with_connection(|mut conn| async move { self.save().fetch(&mut conn).await })
                .await
        }

        pub fn admins() -> Scope<Self> {
            Self::filter("admin", true)
        }

        pub fn tasks(&self) -> Scope<Task> {
            Task::filter("user_id", self.id)
        }

        pub fn completed_tasks(&self) -> Scope<Task> {
            self.tasks().not("completed_at", Value::Null)
        }

        pub fn incomplete_tasks(&self) -> Scope<Task> {
            self.tasks().filter("completed_at", Value::Null)
        }

        pub fn recently_completed(&self) -> Scope<Task> {
            let last_24 = OffsetDateTime::now_utc() - Duration::hours(24);

            self.tasks().filter_gte("completed_at", last_24)
        }
    }

    #[derive(Clone, rum::macros::Model, Debug)]
    #[belongs_to(User)]
    pub struct Task {
        pub id: Option<i64>,
        pub user_id: i64,
        pub name: String,
        pub created_at: OffsetDateTime,
        pub completed_at: Option<OffsetDateTime>,
    }

    impl Task {
        pub fn completed() -> Scope<Self> {
            Task::all().not("completed_at", Value::Null)
        }

        pub fn completed_by_admins() -> Scope<Self> {
            Task::completed()
                .join::<User>()
                .filter(User::column("admin"), true)
        }

        pub fn completed_or_created_by_admins() -> Scope<Self> {
            Task::completed()
                .join::<User>()
                .or(|scope| scope.filter(User::column("admin"), true))
        }

        pub async fn complete(mut self) -> Result<Self, Error> {
            self.completed_at = Some(OffsetDateTime::now_utc());

            let mut conn = Pool::connection().await?;

            Ok(self.save().fetch(&mut conn).await?)
        }
    }
}

use models::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Config::load().await?;
    Logger::init();

    Migrations::flush().await?;
    Migrations::migrate().await?;

    let user = User::create_user("test@test.com").await?;
    for i in 0..3 {
        let name = format!("task_{}", i);
        user.add_task(&name).await?;
    }

    // Get a connection from the pool and use it to execute some queries.
    // Using a closure ensures the connection is returned to the pool as soon
    // as all the queries inside the closure are complete.
    let admins = Pool::pool()
        .with_connection(|mut conn| async move { User::admins().count(&mut conn).await })
        .await?;

    assert_eq!(admins, 0);

    // Checkout a connection from the pool manually.
    let mut conn = Pool::connection().await?;

    let (tasks, completed) = {
        let tasks = user.tasks().count(&mut conn).await?;
        let completed = user.recently_completed().count(&mut conn).await?;

        (tasks, completed)
    };

    assert_eq!(tasks, 3);
    assert_eq!(completed, 0);

    // This will checkout an additional connection from the pool
    // and return it immediately after the future resolves.
    let user = user.make_admin().await?;

    let created_by_admins_or_completed = Task::completed_or_created_by_admins()
        .count(&mut conn)
        .await?;

    assert_eq!(created_by_admins_or_completed, 3);

    let user = user.remove_admin().await?;

    user.complete_all_tasks().await?;

    // Reload user model.
    let user = user.reload().fetch(&mut conn).await?;
    assert_eq!(user.completed_tasks, 3);

    let completed_tasks_count = user.completed_tasks().count(&mut conn).await?;
    assert_eq!(completed_tasks_count, 3);

    let tasks = Task::completed_by_admins().fetch_all(&mut conn).await?;
    assert!(tasks.is_empty());

    let task = Task::first_one().fetch(&mut conn).await?;
    let task = task.complete().await?;

    assert!(task.completed_at.is_some());

    let _users = User::filter("email", ["test@test.com", "joe@test.com"].as_slice())
        .fetch_all(&mut conn).await?;

    Ok(())
}

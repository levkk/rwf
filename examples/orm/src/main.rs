#![allow(dead_code)]
use rum::logging::setup_logging;
use rum::prelude::*;

mod models {
    use rum::model::prelude::*;
    use time::{Duration, OffsetDateTime};

    #[derive(Clone, rum::macros::Model)]
    #[has_many(Task)]
    pub struct User {
        pub id: Option<i64>,
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
                ("name", task_name.to_value()),
                ("user_id", self.id.to_value()),
            ])
            .fetch(&mut conn)
            .await?;

            Ok(task)
        }

        pub async fn complete_all_tasks(&self) -> Result<Vec<Task>, Error> {
            let tasks = Pool::pool()
                .with_transaction(|mut transaction| async move {
                    let _lock = self.tasks().lock().execute(&mut transaction).await?;

                    let tasks = self
                        .tasks()
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

                    transaction.commit().await?;

                    Ok(tasks)
                })
                .await?;

            Ok(tasks)
        }

        pub fn admins(&self) -> Scope<Self> {
            Self::filter("admin", true)
        }

        pub fn tasks(&self) -> Scope<Task> {
            Task::filter("user_id", self.id)
        }

        pub fn recently_completed(&self) -> Scope<Task> {
            let last_24 = OffsetDateTime::now_utc() - Duration::hours(24);

            self.tasks().filter_gte("completed_at", last_24)
        }
    }

    #[derive(Clone, rum::macros::Model)]
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
            Task::completed().or(|scope| scope.join::<User>().filter(User::column("admin"), true))
        }

        pub async fn complete(mut self) -> Result<Self, Error> {
            self.completed_at = Some(OffsetDateTime::now_utc());

            let mut conn = Pool::connection().await?;

            Ok(self.save().fetch(&mut conn).await?)
        }
    }
}

#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>Hey Rum!</h1>"))
    }
}

#[tokio::main]
async fn main() {
    setup_logging();

    // let user = User::create(&[
    //     ("email", "test@test.com".to_value()),
    //     ("admin", false.to_value()),
    // ]);
}

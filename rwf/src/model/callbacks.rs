// #[async_trait]
// pub trait Callbacks: Model {
//     async fn after_save(&self, conn: &tokio_postgres::Client) -> Result<(), Error> {
//         Ok(())
//     }

//     async fn before_save(&self, conn: &tokio_postgres::Client) -> Result<(), Error> {
//         Ok(())
//     }
// }

// struct Test;

// #[async_trait]
// impl Callbacks for Test {}

use super::{Column, Error, Escape, FromRow, Model, Placeholders, Query, ToSql, ToValue};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Update<T> {
    table_name: String,
    primary_key: String,
    pub placeholders: Placeholders,
    columns: Vec<String>,
    marker: PhantomData<T>,
}

impl<T: Model> Update<T> {
    pub fn new(model: T) -> Self {
        let columns = T::column_names()
            .into_iter()
            .filter(|column| column != &T::primary_key())
            .map(|column| column.escape())
            .collect();
        let values = model.values();
        let mut placeholders = Placeholders::new();
        for value in values {
            placeholders.add(&value);
        }

        placeholders.add(&model.id().to_value());

        Self {
            table_name: T::table_name(),
            primary_key: T::primary_key(),
            placeholders,
            columns,
            marker: PhantomData,
        }
    }

    pub async fn save(self, client: &tokio_postgres::Client) -> Result<T, Error> {
        // Query::Update(self)
        // 	.fetch(&client)
        // 	.await?
        todo!()
    }
}

impl<T: FromRow> ToSql for Update<T> {
    fn to_sql(&self) -> String {
        let where_id = format!("id = ${}", self.placeholders.id() - 1);
        let sets = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, column)| format!(r#""{}" = ${}"#, column.escape(), idx + 1))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"UPDATE "{}" SET {} WHERE {} RETURNING *"#,
            self.table_name.escape(),
            sets,
            where_id
        )
    }
}

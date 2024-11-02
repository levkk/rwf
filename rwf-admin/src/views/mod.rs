use crate::models::*;
use rwf::prelude::*;

#[derive(Debug)]
pub struct Records {
    table_name: String,
    selected_columns: Vec<String>,
}

impl Records {
    pub async fn render(&self) -> Result<String, Error> {
        let columns = TableColumn::for_table(&self.table_name).await?;
        let create_columns = columns
            .clone()
            .into_iter()
            .filter(|c| !c.skip())
            .collect::<Vec<_>>();
        let order_by = if columns
            .iter()
            .find(|c| c.column_name == "id")
            .take()
            .is_some()
        {
            "ORDER BY id DESC "
        } else {
            ""
        };
        todo!()
    }
}

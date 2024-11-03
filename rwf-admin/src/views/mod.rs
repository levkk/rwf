use crate::models::*;
use rwf::prelude::*;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Records {
    table_name: String,
    selected_columns: Vec<String>,
}

#[allow(dead_code)]
impl Records {
    pub async fn render(&self) -> Result<String, Error> {
        let columns = TableColumn::for_table(&self.table_name).await?;
        let _create_columns = columns
            .clone()
            .into_iter()
            .filter(|c| !c.skip())
            .collect::<Vec<_>>();
        let _order_by = if columns
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

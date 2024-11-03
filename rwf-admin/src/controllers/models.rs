use rwf::model::{Escape, Row};
use rwf::prelude::*;

use crate::models::*;

#[derive(Default)]
pub struct ModelsController;

#[async_trait]
impl Controller for ModelsController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let tables = Table::load().await?;
        render!("templates/rwf_admin/models.html", "models" => tables)
    }
}

#[derive(Default, macros::PageController)]
pub struct ModelController;

#[async_trait]
impl PageController for ModelController {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        let model = request.query().get::<String>("name");
        let selected_columns = request
            .query()
            .get::<String>("columns")
            .unwrap_or("".to_string())
            .split(",")
            .into_iter()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect::<Vec<_>>();

        if let Some(model) = model {
            let columns = TableColumn::for_table(&model).await?;
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

            let columns = columns
                .into_iter()
                .map(|c| c.column_name)
                .collect::<Vec<_>>();
            let selected_columns = columns
                .clone()
                .into_iter()
                .filter(|c| selected_columns.contains(&c) || selected_columns.is_empty())
                .collect::<Vec<_>>();

            if !columns.is_empty() {
                let table_name = model.clone();
                let rows = Pool::pool()
                    .with_connection(|mut conn| async move {
                        Row::find_by_sql(
                            format!(
                                "SELECT * FROM \"{}\" {}LIMIT 25",
                                table_name.escape(),
                                order_by
                            ),
                            &[],
                        )
                        .fetch_all(&mut conn)
                        .await
                    })
                    .await?;
                let mut data = vec![];
                for row in rows {
                    data.push(row.values()?);
                }

                render!("templates/rwf_admin/model.html",
                    "table_name" => model,
                    "columns" => columns,
                    "rows" => data,
                    "create_columns" => create_columns,
                    "selected_columns" => selected_columns,
                )
            }
        }

        Ok(Response::not_found())
    }
}

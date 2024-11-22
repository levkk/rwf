use rwf::model::{Escape, Row};
use rwf::prelude::*;

use crate::models::*;

#[derive(Default)]
pub struct ModelsController;

#[async_trait]
impl Controller for ModelsController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let tables = Table::load().await?;
        render!("templates/rwf_admin/models.html",
            "title" => "Models | Rust Web Framework",
            "models" => tables
        )
    }
}

#[derive(Default, macros::PageController)]
pub struct ModelController;

#[async_trait]
impl PageController for ModelController {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        let model = request.query().get::<String>("name");
        let page = request.query().get::<i64>("page").unwrap_or(1);
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
                                "SELECT * FROM \"{}\" {}LIMIT 25{}",
                                table_name.escape(),
                                order_by,
                                format!(" OFFSET {}", (page - 1) * 25),
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
                    "title" => format!("{} | Rust Web Framework", model),
                    "table_name" => model,
                    "columns" => columns,
                    "rows" => data,
                    "create_columns" => create_columns,
                    "selected_columns" => selected_columns,
                    "page" => page,
                )
            }
        }

        Ok(Response::not_found())
    }
}

#[derive(Default, macros::PageController)]
pub struct NewModelController;

#[async_trait]
impl PageController for NewModelController {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        let model = request.query().get_required::<String>("name")?;
        let columns = TableColumn::for_table(&model)
            .await?
            .into_iter()
            .filter(|c| !c.skip())
            .collect::<Vec<_>>();

        render!("templates/rwf_admin/model_new.html",
            "title" => format!("New record | {} | Rust Web Framework", model),
            "table_name" => model,
            "columns" => columns,
        )
    }

    async fn post(&self, req: &Request) -> Result<Response, Error> {
        let query = req
            .form_data()?
            .into_iter()
            .filter(|c| c.0 != "rwf_csrf_token");
        let mut columns = vec![];
        let mut values = vec![];
        let mut table_name = vec![];

        for (column, value) in query {
            if column == "rwf_table_name" {
                table_name.push(value.escape());
                continue;
            }

            columns.push(format!("\"{}\"", column.escape()));
            values.push(if value.is_empty() {
                "NULL".to_string()
            } else {
                format!("'{}'", value.escape())
            });
        }

        let table_name = table_name.pop().unwrap();

        let query = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({})",
            table_name,
            columns.join(", "),
            values.join(", ")
        );

        Pool::pool()
            .with_connection(|mut conn| async move { conn.query_cached(&query, &[]).await })
            .await?;

        Ok(Response::new().redirect(format!("/admin/models/model?name={}", table_name)))
    }
}

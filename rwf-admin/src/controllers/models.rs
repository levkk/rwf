use std::collections::HashMap;

use rwf::model::{Escape, Row};
use rwf::prelude::*;

use uuid::Uuid;

#[derive(Clone, macros::Model)]
struct Table {
    table_name: String,
}

impl Table {
    async fn load() -> Result<Vec<Table>, Error> {
        Ok(Pool::pool()
            .with_connection(|mut conn| async move {
                Table::find_by_sql(
                    "
                SELECT
                    relname::text AS table_name
                FROM
                    pg_class
                WHERE
                    pg_table_is_visible(oid)
                    AND reltype != 0
                    AND relname NOT LIKE 'pg_%'
                ORDER BY oid",
                    &[],
                )
                .fetch_all(&mut conn)
                .await
            })
            .await?)
    }
}

#[derive(Clone, macros::Model)]
struct TableColumn {
    table_name: String,
    column_name: String,
    data_type: String,
    column_default: String,
}

impl TableColumn {
    pub async fn for_table(name: &str) -> Result<Vec<TableColumn>, Error> {
        Ok(Pool::pool()
            .with_connection(|mut conn| async move {
                TableColumn::find_by_sql(
                    "SELECT
                table_name::text,
                column_name::text,
                data_type::text,
                COALESCE(column_default::text, '')::text AS column_default
            FROM information_schema.columns
            INNER JOIN pg_class ON table_name = relname
            INNER JOIN pg_namespace ON pg_namespace.oid = pg_class.relnamespace
            WHERE pg_class.relname = $1::text AND pg_table_is_visible(pg_class.oid)
            ORDER BY ordinal_position",
                    &[name.to_value()],
                )
                .fetch_all(&mut conn)
                .await
            })
            .await?
            .into_iter()
            .map(|c| c.transform_default())
            .collect())
    }

    pub fn transform_default(mut self) -> Self {
        if self.column_default == "now()" {
            let format =
                time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
            self.column_default = OffsetDateTime::now_utc().format(format).unwrap();
        } else if self.column_default == "gen_random_uuid()" {
            self.column_default = Uuid::new_v4().to_string();
        } else if self.column_default.ends_with("::character varying") {
            self.column_default = self
                .column_default
                .replace("::character varying", "")
                .replace("'", "");
        } else if self.column_default.ends_with("::jsonb") {
            self.column_default = self.column_default.replace("::jsonb", "").replace("'", "");
        }

        self
    }

    pub fn skip(&self) -> bool {
        if self.column_default.starts_with("nextval(") {
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct ModelsController;

#[async_trait]
impl Controller for ModelsController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let tables = Table::load().await?;
        render!("templates/rwf_admin/models.html", "models" => tables)
    }
}

#[derive(Default)]
pub struct ModelController;

#[async_trait]
impl Controller for ModelController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
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
            let columns = TableColumn::for_table(&model)
                .await?
                .into_iter()
                .filter(|c| {
                    selected_columns.contains(&c.column_name) || selected_columns.is_empty()
                })
                .collect::<Vec<_>>();
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
                    "create_columns" => create_columns)
            }
        }

        Ok(Response::not_found())
    }
}

use rwf::prelude::*;

#[derive(Clone, macros::Model, Debug)]
pub struct TableColumn {
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
    pub column_default: String,
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

#[derive(Clone, macros::Model)]
pub struct Table {
    pub table_name: String,
}

impl Table {
    pub async fn load() -> Result<Vec<Table>, Error> {
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

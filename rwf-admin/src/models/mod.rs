use rwf::prelude::*;

#[derive(Clone, macros::Model, Debug)]
pub struct TableColumn {
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
    pub column_default: String,
    pub is_nullable: bool,
    pub is_required: bool,
    pub placeholder: String,
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
                COALESCE(column_default::text, '')::text AS column_default,
                is_nullable::boolean AS is_nullable,
                (is_nullable::boolean = false AND COALESCE(column_default::text, '') = '')::boolean AS is_required,
                '' AS placeholder
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
            .map(|c| c.transform())
            .collect())
    }

    fn transform(mut self) -> Self {
        let format =
            time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

        if self.column_default == "now()" {
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

        match self.data_type.as_str() {
            "timestamp with time zone" => {
                self.placeholder = OffsetDateTime::now_utc().format(format).unwrap();
            }

            "uuid" => {
                self.placeholder = Uuid::new_v4().to_string();
            }

            _ => (),
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

#[derive(Clone, macros::Model, Serialize)]
pub struct RequestByCode {
    pub count: i64,
    pub code: String,
    #[serde(with = "time::serde::rfc2822")]
    pub created_at: OffsetDateTime,
}

impl RequestByCode {
    pub fn count(minutes: i64) -> Scope<Self> {
        Self::find_by_sql(
            "WITH timestamps AS (
                SELECT date_trunc('minute', now() - (n || ' minute')::interval) AS created_at FROM generate_series(0, $1::bigint) n
            )
            SELECT
                'ok' AS code,
                COALESCE(e2.count, 0) AS count,
                timestamps.created_at AS created_at
            FROM timestamps
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) AS count,
                    DATE_TRUNC('minute', created_at) AS created_at
                FROM rwf_requests
                WHERE
                    created_at BETWEEN timestamps.created_at AND timestamps.created_at + INTERVAL '1 minute'
                    AND code BETWEEN 100 AND 299
                GROUP BY 2
            ) e2 ON true
            UNION ALL
            SELECT
                'warn' AS code,
                COALESCE(e2.count, 0) AS count,
                timestamps.created_at AS created_at
            FROM timestamps
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) AS count,
                    DATE_TRUNC('minute', created_at) AS created_at
                FROM rwf_requests
                WHERE
                    created_at BETWEEN timestamps.created_at AND timestamps.created_at + INTERVAL '1 minute'
                    AND code BETWEEN 300 AND 499
                GROUP BY 2
            ) e2 ON true
            UNION ALL
            SELECT
                'error' AS code,
                COALESCE(e2.count, 0) AS coount,
                timestamps.created_at AS created_at
            FROM timestamps
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) AS count,
                    DATE_TRUNC('minute', created_at) AS created_at
                FROM rwf_requests
                WHERE
                    created_at BETWEEN timestamps.created_at AND timestamps.created_at + INTERVAL '1 minute'
                    AND code BETWEEN 500 AND 599
                GROUP BY 2
            ) e2 ON true
            ORDER BY 3;",
            &[minutes.to_value()],
        )
    }
}

#[derive(Clone, macros::Model, Serialize)]
pub struct RequestsDuration {
    pub duration: f64,
    #[serde(with = "time::serde::rfc2822")]
    pub created_at: OffsetDateTime,
}

impl RequestsDuration {
    pub fn count(minutes: i64) -> Scope<Self> {
        Self::find_by_sql(
            "WITH timestamps AS (
                SELECT date_trunc('minute', now() - (n || ' minute')::interval) AS created_at FROM generate_series(0, $1::bigint) n
            )
            SELECT
                COALESCE(e2.avg, 0.0) AS duration,
                timestamps.created_at AS created_at
            FROM timestamps
            LEFT JOIN LATERAL (
                SELECT avg(duration) AS avg
                FROM rwf_requests
                WHERE created_at BETWEEN timestamps.created_at AND timestamps.created_at + INTERVAL '1 minute'
            ) e2 ON true
            ORDER BY 2", &[minutes.to_value()],
        )
    }
}

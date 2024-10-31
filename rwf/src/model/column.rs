use super::{Escape, ToSql, ToValue, Value};

/// PostgreSQL table column.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    table_name: String,
    column_name: String,
    as_value: Option<Box<Value>>,
}

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_sql())
    }
}

impl ToSql for Column {
    fn to_sql(&self) -> String {
        let as_value = if let Some(ref as_value) = self.as_value {
            format!("{} AS ", as_value.to_sql())
        } else {
            "".to_string()
        };

        if self.table_name.is_empty() {
            format!(r#"{}"{}""#, as_value, self.column_name.escape())
        } else {
            format!(
                r#"{}"{}"."{}""#,
                as_value,
                self.table_name.escape(),
                self.column_name.escape(),
            )
        }
    }
}

impl Column {
    /// Create new table column, given the table name and column name.
    ///
    /// Columns are ideally always fully qualified with the table name
    /// to avoid ambiguous errors.
    pub fn new(table_name: impl ToString, column_name: impl ToString) -> Self {
        Self {
            table_name: table_name.to_string(),
            column_name: column_name.to_string(),
            as_value: None,
        }
    }

    /// Create new table column, given the column name.
    ///
    /// Not fully qualified, so use with care, or you'll get
    /// ambiguous column error when joining, especially with common column
    /// names like "id".
    pub fn name(column_name: impl ToString) -> Self {
        Self::new("", column_name)
    }

    pub fn qualified(&self) -> bool {
        !self.table_name.is_empty()
    }

    pub fn qualify(mut self, table_name: impl ToString) -> Self {
        self.table_name = table_name.to_string();
        self
    }

    pub fn unqualify(mut self) -> Self {
        self.table_name.clear();
        self
    }

    pub fn as_value(mut self, value: impl ToValue) -> Self {
        self.as_value = Some(Box::new(value.to_value()));
        self
    }
}

#[derive(Debug, Clone)]
pub struct Columns {
    pub columns: Vec<Column>,
    table_name: Option<String>,
    exists: bool,
    all: bool,
    count: bool,
}

impl Default for Columns {
    fn default() -> Self {
        Self {
            columns: vec![],
            table_name: None,
            exists: false,
            all: true,
            count: false,
        }
    }
}

impl Columns {
    pub fn pick(columns: &[impl ToColumn]) -> Self {
        Self {
            columns: columns.iter().map(|c| c.to_column()).collect(),
            all: false,
            ..Default::default()
        }
    }

    pub fn picked(&self) -> bool {
        !self.columns.is_empty()
    }

    pub fn table_name(mut self, table_name: impl ToString) -> Self {
        self.table_name = Some(table_name.to_string());
        self
    }

    pub fn exists(mut self) -> Self {
        self.exists = true;
        self
    }

    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }

    pub fn count(mut self) -> Self {
        self.count = true;
        self
    }

    pub fn add_column(mut self, column: impl ToColumn) -> Self {
        self.columns.push(column.to_column());
        self
    }
}

impl ToSql for Columns {
    fn to_sql(&self) -> String {
        if self.exists {
            "COUNT(*) AS count".into()
        } else {
            let mut columns = if self.count {
                vec!["COUNT(*) AS count".to_string()]
            } else {
                vec![]
            };

            if self.columns.is_empty() || self.all {
                if let Some(ref table_name) = self.table_name {
                    columns.push(format!(r#""{}".*"#, table_name));
                } else {
                    columns.push("*".to_string());
                }
            }

            columns.extend(self.columns.iter().map(|column| column.to_sql()));

            columns.join(", ")
        }
    }
}

pub trait ToColumn {
    fn to_column(&self) -> Column;
}

impl ToColumn for String {
    fn to_column(&self) -> Column {
        Column::name(self)
    }
}

impl ToColumn for &str {
    fn to_column(&self) -> Column {
        Column::name(self)
    }
}

impl ToColumn for Column {
    fn to_column(&self) -> Column {
        self.clone()
    }
}

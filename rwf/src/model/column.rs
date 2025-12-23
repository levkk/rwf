//! Represents the database table column.

use super::{Escape, ToSql, ToValue, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Aggregation {
    SUM,
    AVG, 
    MIN,
    MAX,
    COUNT,
    NONE
}

impl Default for Aggregation {
    fn default() -> Self {
        Self::NONE
    }
}

impl std::fmt::Display for Aggregation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SUM => write!(f, "SUM"),
            Self::MIN => write!(f, "MIN"),
            Self::MAX => write!(f, "MAX"),
            Self::AVG => write!(f, "AVG"),
            Self::COUNT => write!(f, "COUNT"),
            Self::NONE => write!(f, "NONE"),
        }
    }
}

pub trait ToAggregation {
    fn to_agg(&self) -> Aggregation;
}

impl ToAggregation for str {
    fn to_agg(&self) -> Aggregation {
        match self {
            "sum" => Aggregation::SUM,
            "Sum" => Aggregation::SUM,
            "SUM" => Aggregation::SUM,
            "min" => Aggregation::MIN,
            "Min" => Aggregation::MIN,
            "MIN" => Aggregation::MIN,
            "max" => Aggregation::MAX,
            "Max" => Aggregation::MAX,
            "MAX" => Aggregation::MAX,
            "avg" => Aggregation::AVG,
            "Avg" => Aggregation::AVG,
            "AVG" => Aggregation::AVG,
            "count" => Aggregation::COUNT,
            "Count" => Aggregation::COUNT,
            "COUNT" => Aggregation::COUNT,
            _ => Aggregation::NONE
        }
    }
}

impl ToAggregation for &str {
    fn to_agg(&self) -> Aggregation {
        (*self).to_agg()
    }
}
impl ToAggregation for String {
    fn to_agg(&self) -> Aggregation {
        self.as_str().to_agg()
    }
}
impl ToAggregation for Aggregation {
    fn to_agg(&self) -> Aggregation {
       *self
    }
}


impl Aggregation {
    pub fn is_none(&self) -> bool {self.eq(&Self::NONE)}
    pub fn is_agg(&self) -> bool {!self.is_none()}
}

impl<T: ToAggregation + Sized> ToAggregation for &T {
    fn to_agg(&self) -> Aggregation {
        (*self).to_agg()
    }
}

/// PostgreSQL table column.
#[derive(Debug, Clone, Hash)]
pub struct Column {
    table_name: String,
    column_name: String,
    as_value: Option<Box<Value>>,
    agg: Aggregation
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        if self.as_value.is_some() == other.as_value.is_none() {
            false
        } else if self.as_value.is_some() && other.as_value.is_some() {
            self.table_name.eq(&other.table_name) && self.column_name.eq(&other.column_name) && self.agg.eq(&other.agg) && self.as_value.as_ref().unwrap().eq(&other.as_value.as_ref().unwrap())

        } else {
            self.table_name.eq(&other.table_name) && self.column_name.eq(&other.column_name) && self.agg.eq(&other.agg)
        }
    }
}
impl Eq for Column {}

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
        let sql = if self.table_name.is_empty() {
            format!(r#"{}"{}""#, as_value, self.column_name.escape())
        } else {
            format!(
                r#"{}"{}"."{}""#,
                as_value,
                self.table_name.escape(),
                self.column_name.escape(),
            )
        };
        if self.agg.is_none() {
            sql
        } else {
            format!(r#"{}({}) as "{}""#, self.agg, sql, self.column_name.escape())
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
            agg: Aggregation::default(),
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
    pub fn agg(mut self, value: impl ToAggregation) -> Self {
        self.agg = value.to_agg();
        self
    }
    pub fn aggregation(&self) -> &Aggregation {&self.agg}

    pub fn get_name(&self) -> &str {self.column_name.as_str()}
    pub fn get_table_name(&self) -> &str {self.table_name.as_str()}
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
            self.columns.iter().map(|col| col.to_sql()).collect::<Vec<_>>().join(", ")

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
impl<T: ToColumn> ToColumn for &T {
    fn to_column(&self) -> Column {
        (**self).to_column()
    }
}   

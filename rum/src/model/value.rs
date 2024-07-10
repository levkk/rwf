use bytes::BytesMut;
use time::{OffsetDateTime, PrimitiveDateTime};
use tokio_postgres::types::{to_sql_checked, IsNull, Type};

use std::ops::Range;

use super::{Column, Error, Escape, ToSql};

/// A value that can be converted to and from the database.
///
/// This includes primitive types like [`String`] and [`i64`],
/// and expands all the way to placeholders in prepared statements, e.g. `$1`,
/// and table columns.
#[derive(Debug, Clone)]
pub enum Value {
    /// Regular string, e.g. `'hello'`.
    String(String),
    /// Integer, e.g. `123`.
    Integer(i64),
    /// Floating point number, e.g. `3.14`.
    Float(f64),
    /// Timestamp with time zone speficiation.
    TimestampT(OffsetDateTime),
    /// Timestamp without time zone.
    Timestamp(PrimitiveDateTime),
    /// List (Postgres array) of values, e.g. `{1, 2, 3}`.
    List(Vec<Value>),
    /// Tuple (also known as "record") of values, e.g. `(1, 2, 3)`.
    Record(Box<Value>),
    /// Placeholder in a prepared statemnt, e.g. `$1`.
    Placeholder(i32),
    /// Range of values, e.g. `BETWEEN 5 AND 25`.
    Range((Box<Value>, Box<Value>)),
    /// Table column, e.g. `"users"."id"`.
    Column(Column),
    /// Nullable value of any of the above (which make sense).
    Optional(Box<Option<Value>>),
}

impl Value {
    /// Create a new value.
    ///
    /// The input needs to implement [`ToValue`] trait. Implementations for standard and common
    /// Rust types are provided.
    pub fn new(value: impl ToValue) -> Self {
        value.to_value()
    }
}

/// Convert anything to a [`Value`].
///
/// Implementation for many common types are provided, e.g. [`String`], [`i64`], [`OffsetDateTime`], and more.
pub trait ToValue {
    fn to_value(&self) -> Value;
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl ToValue for Option<String> {
    fn to_value(&self) -> Value {
        Value::Optional(Box::new(self.as_ref().map(|v| v.to_value())))
    }
}

impl ToValue for Option<&str> {
    fn to_value(&self) -> Value {
        Value::Optional(Box::new(self.map(|v| v.to_value())))
    }
}

impl ToValue for &str {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToValue for i64 {
    fn to_value(&self) -> Value {
        Value::Integer(*self)
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Value::Float(*self)
    }
}

impl ToValue for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl ToValue for &[&str] {
    fn to_value(&self) -> Value {
        Value::List(self.iter().map(|v| v.to_value()).collect::<Vec<_>>())
    }
}

impl ToValue for &[i64] {
    fn to_value(&self) -> Value {
        Value::List(self.iter().map(|v| v.to_value()).collect::<Vec<_>>())
    }
}

impl ToValue for Column {
    fn to_value(&self) -> Value {
        Value::Column(self.clone())
    }
}

impl ToValue for Range<i64> {
    fn to_value(&self) -> Value {
        Value::Range((
            Box::new(self.start.to_value()),
            Box::new(self.end.to_value()),
        ))
    }
}

impl tokio_postgres::types::ToSql for Value {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        use std::ops::Deref;
        match self {
            Value::String(string) => string.to_sql(ty, out),
            Value::Integer(integer) => integer.to_sql(ty, out),
            Value::Float(float) => float.to_sql(ty, out),
            Value::TimestampT(timestamp) => timestamp.to_sql(ty, out),
            Value::Timestamp(timestamp) => timestamp.to_sql(ty, out),
            Value::List(values) => values.to_sql(ty, out),
            Value::Optional(value) => {
                if let Some(value) = value.deref() {
                    tokio_postgres::types::ToSql::to_sql(&value, ty, out)
                } else {
                    return Ok(IsNull::Yes);
                }
            }
            value => return Err(Error::OrmSerializationError(value.clone()).boxed()),
        }
    }

    fn accepts(_ty: &Type) -> bool {
        // Handled by to_sql.
        true
    }

    to_sql_checked!();
}

impl ToSql for Value {
    fn to_sql(&self) -> String {
        use Value::*;

        match self {
            Value::String(string) => format!("'{}'", string.escape()),
            Integer(integer) => integer.to_string(),
            Float(float) => float.to_string(),
            Placeholder(number) => format!("${}", number),
            Range((a, b)) => format!("BETWEEN {} AND {}", a.to_sql(), b.to_sql()),
            List(values) => format!(
                "{{{}}}",
                values
                    .iter()
                    .map(|value| value.to_sql())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Column(column) => column.to_sql(),
            _ => todo!(),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_range_i64() {
        let value = Value::new(1_i64..25);
        assert_eq!(value.to_sql(), "BETWEEN 1 AND 25");
    }
}

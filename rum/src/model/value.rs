use bytes::BytesMut;
use time::{OffsetDateTime, PrimitiveDateTime};
use tokio_postgres::{
    types::{to_sql_checked, IsNull, Type},
    Client,
};

use std::ops::{Deref, Range};

use super::{Error, Escape, ToSql};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    TimestampT(OffsetDateTime),
    Timestamp(PrimitiveDateTime),
    List(Vec<Value>),
    Placeholder(i32),
    Range((Box<Value>, Box<Value>)),
}

impl Value {
    /// Create a new value.
    pub fn new(value: impl ToValue) -> Self {
        value.to_value()
    }
}

pub trait ToValue {
    fn to_value(&self) -> Value;
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
        match self {
            Value::String(string) => string.to_sql(ty, out),
            Value::Integer(integer) => integer.to_sql(ty, out),
            Value::Float(float) => float.to_sql(ty, out),
            Value::TimestampT(timestamp) => timestamp.to_sql(ty, out),
            Value::Timestamp(timestamp) => timestamp.to_sql(ty, out),
            Value::List(values) => values.to_sql(ty, out),
            value => return Err(Error::OrmSerializationError(value.clone()).boxed()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        // Handled by to_sql.
        true
    }

    to_sql_checked!();
}

#[derive(Debug)]
pub struct Values {
    values: Vec<Value>,
}

impl ToSql for Values {
    fn to_sql(&self) -> String {
        self.values
            .iter()
            .map(|value| value.to_sql())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Values {
    pub fn new(values: &[Value]) -> Values {
        Values {
            values: values.to_vec(),
        }
    }
}

impl From<&[Value]> for Values {
    fn from(values: &[Value]) -> Self {
        Values {
            values: values.to_vec(),
        }
    }
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
        let _value = Value::new(1_i64..25);
    }
}

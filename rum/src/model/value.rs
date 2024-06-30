use bytes::BytesMut;
use time::{OffsetDateTime, PrimitiveDateTime};
use tokio_postgres::{
    types::{to_sql_checked, Format, IsNull, Type},
    Client,
};

use super::ToSql;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    TimestampT(OffsetDateTime),
    Timestamp(PrimitiveDateTime),
    List(Vec<Value>),
    Placeholder(i32),
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
            // Value::TimestampT(timestampt) => timestampt.to_sql(ty, out),
            _ => todo!(),
        }
    }

    fn accepts(ty: &Type) -> bool {
        todo!()
    }

    to_sql_checked!();
}

impl ToSql for Value {
    fn to_sql(&self) -> String {
        use Value::*;

        match self {
            Value::String(string) => format!("'{}'", string),
            Integer(integer) => integer.to_string(),
            Float(float) => float.to_string(),
            Placeholder(number) => format!("${}", number),
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

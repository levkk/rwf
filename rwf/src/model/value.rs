//! Handles conversions between database types and Rust types.
use bytes::BytesMut;
use time::{OffsetDateTime, PrimitiveDateTime};
use tokio_postgres::types::{to_sql_checked, IsNull, Type};
use uuid::Uuid;

use std::{hash::Hash, net::IpAddr, ops::RangeInclusive};

use super::{Column, Error, Escape, ToSql};

/// A value that can be converted to and from the database.
///
/// This includes primitive types like [`String`] and [`i64`],
/// and expands all the way to placeholders in prepared statements, e.g. `$1`,
/// and table columns.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `VARCHAR`, `TEXT`, regular string, e.g. `'hello'`.
    String(String),
    /// Default integer type used by the ORM, i.e. `BIGINT`.
    Integer(i64),
    /// `BIGINT`, `i64`.
    BigInt(i64),
    /// `INTEGER`, `i32`.
    Int(i32),
    /// `SMALLINT`, `i16`.
    SmallInt(i16),
    /// `DOUBLE PRECISION`, `f64`.
    Float(f64),
    /// `REAL`, `f32`
    Real(f32),
    /// `BOOL`, `bool`.
    Boolean(bool),
    /// `TIMESTAMP WITH TIME ZONE`
    TimestampT(OffsetDateTime),
    /// `TIMESTAMP`
    Timestamp(PrimitiveDateTime),
    /// `INET`
    IpAddr(IpAddr),
    /// `UUID`
    Uuid(Uuid),
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
    /// `JSON`, `JSONB`.
    Json(serde_json::Value),
    /// Nullable value of any of the above (which make sense).
    Optional(Box<Option<Value>>),
    /// A database function, e.g. `now()`
    Function((String, Vec<Value>)),
    /// `NULL`.
    Null,
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Value")
    }
}
impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_sql().hash(state);
    }
}

impl Value {
    /// Create a new value.
    ///
    /// The input needs to implement [`ToValue`] trait. Implementations for standard and common
    /// Rust types are provided.
    pub fn new(value: impl ToValue) -> Self {
        value.to_value()
    }

    /// Check if this value is null. Null values are treated special by the ORM, e.g.,
    /// when doing equality checks.
    pub fn is_null(&self) -> bool {
        match self {
            Value::Optional(value) => value.is_none(),
            Value::Null => true,
            _ => false,
        }
    }

    /// Extract the value from an optional value. Will panic if the value is `None`.
    pub fn exists(self) -> Value {
        match self {
            Value::Optional(value) => value.unwrap(),
            value => value,
        }
    }

    /// Create a database function call without arguments. This can be used
    /// to safely execute functions from user-supplied data without worrying SQL injection attacks.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::model::{Value, ToSql};
    /// let now = Value::function("now");
    ///
    /// assert_eq!(now.to_sql(), r#""now"()"#);
    /// ```
    pub fn function(name: impl ToString) -> Self {
        Self::Function((name.to_string(), vec![]))
    }

    /// Checks if this value is a placeholder, as opposed to an actual value.
    pub fn placeholder(&self) -> bool {
        match self {
            Value::Placeholder(_) => true,
            _ => false,
        }
    }
}

/// Convert a Rust type to a [`Value`]. Implementation for many common types
/// are provided, e.g. [`String`], [`i64`], [`OffsetDateTime`], and more.
pub trait ToValue {
    /// Convert a Rust type to a [`Value`]. Data types have to have their own enum variant. Add one
    /// if your data type is missing (and submit a [PR](https://github.com/levkk/rwf/pulls)).
    fn to_value(&self) -> Value;
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl ToValue for &String {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
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

impl ToValue for i32 {
    fn to_value(&self) -> Value {
        Value::Int(*self)
    }
}

impl ToValue for i16 {
    fn to_value(&self) -> Value {
        Value::SmallInt(*self)
    }
}

impl ToValue for Option<i64> {
    fn to_value(&self) -> Value {
        Value::Optional(Box::new(self.as_ref().map(|v| v.to_value())))
    }
}

impl ToValue for Option<i32> {
    fn to_value(&self) -> Value {
        Value::Optional(Box::new(self.as_ref().map(|v| v.to_value())))
    }
}

impl ToValue for Option<i16> {
    fn to_value(&self) -> Value {
        Value::Optional(Box::new(self.as_ref().map(|v| v.to_value())))
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Value::Float(*self)
    }
}

impl ToValue for f32 {
    fn to_value(&self) -> Value {
        Value::Real(*self)
    }
}

impl ToValue for IpAddr {
    fn to_value(&self) -> Value {
        Value::IpAddr(self.clone())
    }
}

impl ToValue for Option<IpAddr> {
    fn to_value(&self) -> Value {
        Value::Optional(Box::new(self.as_ref().map(|v| v.to_value())))
    }
}

impl ToValue for Uuid {
    fn to_value(&self) -> Value {
        Value::Uuid(self.clone())
    }
}

impl ToValue for Option<Uuid> {
    fn to_value(&self) -> Value {
        Value::Optional(Box::new(self.as_ref().map(|v| v.to_value())))
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

impl ToValue for &[i32] {
    fn to_value(&self) -> Value {
        Value::List(self.iter().map(|v| v.to_value()).collect::<Vec<_>>())
    }
}

impl ToValue for &[i16] {
    fn to_value(&self) -> Value {
        Value::List(self.iter().map(|v| v.to_value()).collect::<Vec<_>>())
    }
}

impl ToValue for &[f32] {
    fn to_value(&self) -> Value {
        Value::List(self.iter().map(|v| v.to_value()).collect::<Vec<_>>())
    }
}

impl ToValue for &[f64] {
    fn to_value(&self) -> Value {
        Value::List(self.iter().map(|v| v.to_value()).collect::<Vec<_>>())
    }
}

impl ToValue for Column {
    fn to_value(&self) -> Value {
        Value::Column(self.clone())
    }
}

impl ToValue for RangeInclusive<i64> {
    fn to_value(&self) -> Value {
        Value::Range((
            Box::new(self.start().to_value()),
            Box::new(self.end().to_value()),
        ))
    }
}

impl ToValue for RangeInclusive<i32> {
    fn to_value(&self) -> Value {
        Value::Range((
            Box::new(self.start().to_value()),
            Box::new(self.end().to_value()),
        ))
    }
}

impl ToValue for RangeInclusive<i16> {
    fn to_value(&self) -> Value {
        Value::Range((
            Box::new(self.start().to_value()),
            Box::new(self.end().to_value()),
        ))
    }
}

impl ToValue for &[Value] {
    fn to_value(&self) -> Value {
        Value::List(self.to_vec())
    }
}

impl ToValue for serde_json::Value {
    fn to_value(&self) -> Value {
        match self {
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(n) = n.as_i64() {
                    return Value::Integer(n);
                }
                if let Some(n) = n.as_f64() {
                    return Value::Float(n);
                }
                panic!("json number not parasable")
            }
            v => Value::Json(v.clone()),
        }
    }
}

impl ToValue for OffsetDateTime {
    fn to_value(&self) -> Value {
        Value::TimestampT(*self)
    }
}

impl ToValue for Option<OffsetDateTime> {
    fn to_value(&self) -> Value {
        Value::Optional(Box::new(self.as_ref().map(|v| v.to_value())))
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Value::Boolean(*self)
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

            // Rust default number is an i32.
            // If the field is a bigint, this will automatically cast it.
            Value::Int(integer) => match ty {
                &Type::INT8 => (*integer as i64).to_sql(ty, out),
                _ => integer.to_sql(ty, out),
            },
            Value::BigInt(integer) => integer.to_sql(ty, out),
            Value::SmallInt(integer) => integer.to_sql(ty, out),
            Value::Float(float) => float.to_sql(ty, out),
            Value::Real(float) => float.to_sql(ty, out),
            Value::Boolean(b) => b.to_sql(ty, out),
            Value::TimestampT(timestamp) => timestamp.to_sql(ty, out),
            Value::Timestamp(timestamp) => timestamp.to_sql(ty, out),
            Value::IpAddr(ip) => ip.to_sql(ty, out),
            Value::Uuid(uuid) => uuid.to_sql(ty, out),
            Value::List(values) => values.to_sql(ty, out),
            Value::Json(json) => json.to_sql(ty, out),
            Value::Optional(value) => {
                if let Some(value) = value.deref() {
                    tokio_postgres::types::ToSql::to_sql(&value, ty, out)
                } else {
                    return Ok(IsNull::Yes);
                }
            }
            Value::Null => return Ok(IsNull::Yes),
            value => return Err(Error::OrmSerializationError(value.clone()).boxed()),
        }
    }

    fn accepts(_ty: &Type) -> bool {
        // Handled by to_sql.
        true
    }

    to_sql_checked!();
}

impl<'a> tokio_postgres::types::FromSql<'a> for Value {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        match ty {
            &Type::BOOL => Ok(Value::Boolean(bool::from_sql(ty, raw)?)),
            &Type::INT8 => Ok(Value::Integer(i64::from_sql(ty, raw)?)),
            &Type::INT4 => Ok(Value::Int(i32::from_sql(ty, raw)?)),
            &Type::INT2 => Ok(Value::SmallInt(i16::from_sql(ty, raw)?)),
            &Type::TEXT | &Type::VARCHAR => Ok(Value::String(String::from_sql(ty, raw)?)),
            &Type::JSON | &Type::JSONB => Ok(Value::Json(serde_json::Value::from_sql(ty, raw)?)),
            &Type::FLOAT4 => Ok(Value::Real(f32::from_sql(ty, raw)?)),
            &Type::FLOAT8 => Ok(Value::Float(f64::from_sql(ty, raw)?)),
            &Type::INET => Ok(Value::IpAddr(IpAddr::from_sql(ty, raw)?)),
            &Type::TIMESTAMPTZ => Ok(Value::TimestampT(OffsetDateTime::from_sql(ty, raw)?)),
            &Type::TIMESTAMP => Ok(Value::Timestamp(PrimitiveDateTime::from_sql(ty, raw)?)),
            &Type::UUID => Ok(Value::Uuid(Uuid::from_sql(ty, raw)?)),

            ty => todo!("unimplemented conversion from {:?} to rust", ty),
        }
    }

    #[allow(unused_variables)]
    fn from_sql_null(ty: &Type) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(Value::Null)
    }

    #[allow(unused_variables)]
    fn accepts(ty: &Type) -> bool {
        true
    }
}

impl ToSql for Value {
    fn to_sql(&self) -> String {
        use Value::*;

        match self {
            Value::String(string) => format!("'{}'", string.escape()),
            Integer(integer) => integer.to_string(),
            Int(integer) => integer.to_string(),
            BigInt(integer) => integer.to_string(),
            SmallInt(integer) => integer.to_string(),
            Float(float) => float.to_string(),
            Real(float) => float.to_string(),
            IpAddr(ip) => ip.to_string(),
            Uuid(uuid) => uuid.to_string(),
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
            Value::Json(value) => format!(
                "'{}'::jsonb",
                serde_json::to_string(value)
                    .unwrap_or("".into())
                    .replace("'", ""),
            ),
            Value::Optional(value) => match value.as_ref() {
                Some(value) => value.to_sql(),
                None => "NULL".to_string(),
            },
            Column(column) => column.to_sql(),
            Function((name, args)) => format!(
                r#""{}"({})"#,
                name.escape().to_lowercase(),
                args.into_iter()
                    .map(|v| v.to_sql())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Null => "NULL".to_string(),
            value => todo!("to_sql not implemented for {:?}", value),
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

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::SmallInt(value)
    }
}

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        use serde_json::value::Number;
        match value {
            Value::Integer(i) => serde_json::Value::Number(i.into()),
            Value::Int(i) => serde_json::Value::Number(i.into()),
            Value::BigInt(i) => serde_json::Value::Number(i.into()),
            Value::SmallInt(i) => serde_json::Value::Number(i.into()),
            Value::String(s) => serde_json::Value::String(s),
            Value::Float(f) => serde_json::Value::Number(Number::from_f64(f).unwrap()),
            Value::Real(f) => serde_json::Value::Number(Number::from_f64(f as f64).unwrap()),
            Value::Json(json) => json,
            Value::IpAddr(ip) => serde_json::Value::String(ip.to_string()),
            Value::Uuid(uuid) => serde_json::Value::String(uuid.to_string()),
            Value::Optional(value) => match *value {
                Some(value) => value.into(),
                None => serde_json::Value::Null,
            },
            Value::Boolean(value) => serde_json::Value::Bool(value),
            Value::Timestamp(timestamp) => {
                use time::format_description::well_known::Rfc2822;
                serde_json::Value::String(timestamp.format(&Rfc2822).unwrap())
            }
            Value::TimestampT(timestamp) => {
                use time::format_description::well_known::Rfc2822;
                serde_json::Value::String(timestamp.format(&Rfc2822).unwrap())
            }
            Value::List(list) => {
                let mut values = vec![];
                for v in list {
                    values.push(v.into());
                }
                serde_json::Value::Array(values)
            }
            Value::Record(_) => serde_json::Value::Null,
            Value::Column(_) => serde_json::Value::Null,
            Value::Function(_) => serde_json::Value::Null,
            Value::Null => serde_json::Value::Null,
            Value::Placeholder(_) => serde_json::Value::Null,
            Value::Range(_) => serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_range_i64() {
        let value = Value::new(1_i64..=25);
        assert_eq!(value.to_sql(), "BETWEEN 1 AND 25");
    }

    #[test]
    fn test_function_args() {
        let value = Value::Function(("lower".into(), vec!["my string".to_value()]));

        assert_eq!(value.to_sql(), r#""lower"('my string')"#);
    }
}

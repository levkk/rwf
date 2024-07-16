use super::Error;

use std::cmp::Ordering;
use std::collections::HashMap;

/// A constant value, e.g. `5` or `"hello world"`.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Hash(HashMap<String, Value>),
    Null,
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => i1.partial_cmp(i2),
            (Value::Integer(i1), Value::Float(f2)) => (*i1 as f64).partial_cmp(f2),
            (Value::Float(f1), Value::Integer(i2)) => f1.partial_cmp(&(*i2 as f64)),
            (Value::Float(f1), Value::Float(f2)) => f1.partial_cmp(f2),
            (Value::String(s1), Value::String(s2)) => s1.partial_cmp(s2),
            (Value::Boolean(b1), Value::Boolean(b2)) => b1.partial_cmp(b2),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    write!(f, "{}", v)?;
                    if i < l.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Value::Hash(h) => {
                write!(f, "{{")?;
                for (i, (k, v)) in h.iter().enumerate() {
                    write!(f, "{}: {}", k, v)?;
                    if i < h.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            Value::Null => write!(f, "null"),
        }
    }
}

impl Value {
    /// If the value, when evaluated in the context of a `if` statement
    /// would result in the `if` statement being executed.
    ///
    /// e.g. `<% if 5 %>five is true<% end %>`
    /// would output "five is true" since `5` is truthy.
    pub fn truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            Value::List(list) => !list.is_empty(),
            Value::Hash(hash) => !hash.is_empty(),
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 + i2),
            (Value::Integer(i1), Value::Float(f2)) => Value::Float(*i1 as f64 + f2),
            (Value::Float(f1), Value::Integer(i2)) => Value::Float(f1 + *i2 as f64),
            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 + f2),
            (Value::String(s1), Value::String(s2)) => Value::String(format!("{}{}", s1, s2)),
            (Value::String(s1), Value::Integer(i2)) => Value::String(format!("{}{}", s1, i2)),
            (Value::Integer(i1), Value::String(s2)) => Value::String(format!("{}{}", i1, s2)),
            (Value::String(s1), Value::Float(f2)) => Value::String(format!("{}{}", s1, f2)),
            (Value::Float(f1), Value::String(s2)) => Value::String(format!("{}{}", f1, s2)),
            (Value::List(list), other) => {
                let mut list = list.clone();
                list.push(other.clone());
                Value::List(list)
            }
            (value, Value::List(list)) => {
                let mut list = vec![value.clone()];
                list.extend(list.clone());
                Value::List(list)
            }
            _ => Value::Null,
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 - i2),
            (Value::Integer(i1), Value::Float(f2)) => Value::Float(*i1 as f64 - f2),
            (Value::Float(f1), Value::Integer(i2)) => Value::Float(f1 - *i2 as f64),
            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 - f2),
            (Value::String(s1), Value::String(s2)) => Value::String(s1.replace(s2, "").to_string()),
            (Value::List(list), other) => {
                let mut list = list.clone();
                list.retain(|v| v != other);
                Value::List(list)
            }
            _ => Value::Null,
        }
    }

    pub fn div(&self, other: &Self) -> Self {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 / i2),
            (Value::Integer(i1), Value::Float(f2)) => Value::Float(*i1 as f64 / f2),
            (Value::Float(f1), Value::Integer(i2)) => Value::Float(f1 / *i2 as f64),
            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 / f2),
            _ => Value::Null,
        }
    }

    pub fn mul(&self, other: &Self) -> Self {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 * i2),
            (Value::Integer(i1), Value::Float(f2)) => Value::Float(*i1 as f64 * f2),
            (Value::Float(f1), Value::Integer(i2)) => Value::Float(f1 * *i2 as f64),
            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 * f2),
            (Value::String(s1), Value::Integer(i1)) => Value::String(s1.repeat(*i1 as usize)),
            (Value::Integer(i1), Value::String(s1)) => Value::String(s1.repeat(*i1 as usize)),
            (Value::List(list), Value::Integer(i1)) => {
                let mut list = list.clone();
                let mut new_list = vec![];
                for _ in 0..*i1 {
                    new_list.extend(list.clone());
                }
                Value::List(new_list)
            }
            _ => Value::Null,
        }
    }

    pub fn call(&self, method_name: &str) -> Self {
        match self {
            Value::Integer(value) => match method_name {
                "abs" => Value::Integer((*value).abs()),
                "to_string" | "to_s" => Value::String(value.to_string()),
                "to_f" | "to_float" => Value::Float(*value as f64),
                _ => Value::Null,
            },

            Value::Float(value) => match method_name {
                "abs" => Value::Float(value.abs()),
                "ceil" => Value::Float(value.ceil()),
                "floor" => Value::Float(value.floor()),
                "round" => Value::Float(value.round()),
                "to_string" => Value::String(value.to_string()),
                "to_i" | "to_integer" => Value::Integer(*value as i64),
                _ => Value::Null,
            },

            Value::String(value) => match method_name {
                "to_uppercase" | "upcase" => Value::String(value.to_uppercase()),
                "to_lowercase" | "downcase" => Value::String(value.to_lowercase()),
                "trim" => Value::String(value.trim().to_string()),
                _ => Value::Null,
            },

            Value::Hash(hash) => match method_name {
                "keys" => Value::List(hash.keys().map(|k| Value::String(k.clone())).collect()),
                "values" => Value::List(hash.values().cloned().collect()),
                key => match hash.get(key) {
                    Some(value) => value.clone(),
                    None => Value::Null,
                },
            },

            _ => Value::Null,
        }
    }
}

pub trait ToValue: Clone {
    fn to_value(&self) -> Result<Value, Error>;
}

impl ToValue for String {
    fn to_value(&self) -> Result<Value, Error> {
        Ok(Value::String(self.clone()))
    }
}

impl ToValue for &str {
    fn to_value(&self) -> Result<Value, Error> {
        Ok(Value::String(self.to_string()))
    }
}

macro_rules! impl_integer {
    ($ty:ty) => {
        impl ToValue for $ty {
            fn to_value(&self) -> Result<Value, Error> {
                Ok(Value::Integer(*self as i64))
            }
        }
    };
}

impl_integer!(i64);
impl_integer!(i32);
impl_integer!(i16);
impl_integer!(i8);
impl_integer!(u64); // Could very much overflow
impl_integer!(u32);
impl_integer!(u16);
impl_integer!(u8);

impl ToValue for f64 {
    fn to_value(&self) -> Result<Value, Error> {
        Ok(Value::Float(*self))
    }
}

impl ToValue for f32 {
    fn to_value(&self) -> Result<Value, Error> {
        Ok(Value::Float(*self as f64))
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Result<Value, Error> {
        Ok(Value::Boolean(*self))
    }
}

macro_rules! impl_list {
    ($ty:ty) => {
        impl ToValue for Vec<$ty> {
            fn to_value(&self) -> Result<Value, Error> {
                let mut values = vec![];
                for v in self.iter() {
                    values.push(v.to_value()?);
                }
                Ok(Value::List(values))
            }
        }

        impl ToValue for &[$ty] {
            fn to_value(&self) -> Result<Value, Error> {
                let mut values = vec![];
                for v in self.iter() {
                    values.push(v.to_value()?);
                }
                Ok(Value::List(values))
            }
        }
    };
}

impl_list!(f64);
impl_list!(i64);
impl_list!(&str);

impl ToValue for Value {
    fn to_value(&self) -> Result<Value, Error> {
        Ok(self.clone())
    }
}

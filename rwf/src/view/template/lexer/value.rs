//! The basic building block of our template language: the value.
//! All values like floats, integers, strings, structs, lists, hashes, etc.
//! are represented using the value.
//!
//! This allows operations across data types, like multiplying lists by integers,
//! or accessing hash keys.
use super::{super::Context, Error};

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::model::Model;

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
    Interpreter,
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
            Value::Interpreter => write!(f, "global"),
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
            Value::Interpreter => true,
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
                let mut new = vec![value.clone()];
                new.extend(list.clone());
                Value::List(new)
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
                let list = list.clone();
                let mut new_list = vec![];
                for _ in 0..*i1 {
                    new_list.extend(list.clone());
                }
                Value::List(new_list)
            }
            (a, b) => {
                println!("Cannot multiply {:?} and {:?}", a, b);
                Value::Null
            }
        }
    }

    pub fn call(
        &self,
        method_name: &str,
        args: &[Value],
        _content: &Context,
    ) -> Result<Self, Error> {
        Ok(match self {
            Value::Integer(value) => match method_name {
                "abs" => Value::Integer((*value).abs()),
                "to_string" | "to_s" => Value::String(value.to_string()),
                "to_f" | "to_float" => Value::Float(*value as f64),
                "times" => {
                    let mut list = vec![];
                    for i in 0..*value {
                        list.push(Value::Integer(i));
                    }
                    Value::List(list)
                }
                method_name => return Err(Error::UnknownMethod(method_name.into())),
            },

            Value::Float(value) => match method_name {
                "abs" => Value::Float(value.abs()),
                "ceil" => Value::Float(value.ceil()),
                "floor" => Value::Float(value.floor()),
                "round" => Value::Float(value.round()),
                "to_string" | "to_s" => Value::String(value.to_string()),
                "to_i" | "to_integer" => Value::Integer(*value as i64),
                _ => return Err(Error::UnknownMethod(method_name.into())),
            },

            Value::String(value) => match method_name {
                "to_uppercase" | "upcase" => Value::String(value.to_uppercase()),
                "to_lowercase" | "downcase" => Value::String(value.to_lowercase()),
                "trim" => Value::String(value.trim().to_string()),
                _ => return Err(Error::UnknownMethod(method_name.into())),
            },

            Value::List(list) => match method_name.parse::<i64>() {
                Ok(index) => match list.get(index as usize) {
                    Some(value) => value.clone(),
                    None => Value::Null,
                },

                Err(_) => match method_name {
                    "enumerate" => Value::List(
                        list.iter()
                            .enumerate()
                            .map(|(i, v)| Value::List(vec![Value::Integer(i as i64), v.clone()]))
                            .collect(),
                    ),

                    // TODO: doesn't work
                    "flatten" => {
                        let mut new_list = vec![];
                        for value in list.clone().into_iter() {
                            match value {
                                Value::List(_) => new_list.extend(value.flatten().to_vec()),
                                _ => new_list.push(value.clone()),
                            }
                        }

                        Value::List(new_list)
                    }

                    "reverse" | "rev" => {
                        Value::List(list.clone().into_iter().rev().collect::<Vec<_>>())
                    }

                    _ => return Err(Error::UnknownMethod(method_name.into())),
                },
            },

            Value::Hash(hash) => match method_name {
                "keys" => Value::List(hash.keys().map(|k| Value::String(k.clone())).collect()),
                "values" => Value::List(hash.values().cloned().collect()),
                "iter" => Value::List(
                    hash.keys()
                        .cloned()
                        .into_iter()
                        .zip(hash.values().cloned())
                        .map(|(k, v)| Value::List(vec![Value::String(k), v]))
                        .collect::<Vec<_>>(),
                ),
                key => match hash.get(key) {
                    Some(value) => value.clone(),
                    None => Value::Null,
                },
            },

            Value::Interpreter => match method_name {
                "encrypt_number" => match &args {
                    &[Value::Integer(n)] => match crate::crypto::encrypt_number(*n) {
                        Ok(n) => Value::String(n),
                        Err(_) => Value::Null,
                    },
                    _ => Value::Null,
                },

                "decrypt_number" => match &args {
                    &[Value::String(n)] => match crate::crypto::decrypt_number(&n) {
                        Ok(n) => Value::Integer(n),
                        Err(_) => Value::Null,
                    },
                    _ => Value::Null,
                },

                "rwf_head" => Value::String(include_str!("../head.html").to_string()),
                _ => return Err(Error::UnknownMethod(method_name.into())),
            },

            _ => return Err(Error::UnknownMethod(method_name.into())),
        })
    }

    pub fn flatten(self) -> Value {
        match self {
            Value::List(list) => {
                let mut new_list = vec![];
                for value in list {
                    new_list.push(value.flatten());
                }

                Value::List(new_list)
            }

            value => Value::List(vec![value]),
        }
    }

    pub fn to_vec(self) -> Vec<Value> {
        match self {
            Value::List(list) => list,
            value => vec![value],
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            value => format!("{:?}", value),
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

impl ToValue for Option<i64> {
    fn to_value(&self) -> Result<Value, Error> {
        match self {
            Some(i) => i.to_value(),
            None => Ok(Value::Null),
        }
    }
}

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
impl_list!(String);
impl_list!(Value);

impl ToValue for Value {
    fn to_value(&self) -> Result<Value, Error> {
        Ok(self.clone())
    }
}

impl TryInto<serde_json::Value> for Value {
    type Error = Error;

    fn try_into(self) -> Result<serde_json::Value, Self::Error> {
        use serde_json::value::Number;
        match self {
            Value::Integer(i) => Ok(serde_json::Value::Number(i.into())),
            Value::Float(f) => Ok(serde_json::Value::Number(Number::from_f64(f).unwrap())),
            Value::String(s) => Ok(serde_json::Value::String(s)),
            Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
            Value::List(l) => {
                let mut list = vec![];
                for v in l {
                    list.push(v.try_into()?);
                }
                Ok(serde_json::Value::Array(list))
            }
            Value::Hash(h) => {
                let mut hash = serde_json::Map::new();
                for (k, v) in h {
                    hash.insert(k, v.try_into()?);
                }
                Ok(serde_json::Value::Object(hash))
            }
            Value::Null => Ok(serde_json::Value::Null),
            Value::Interpreter => Ok(serde_json::Value::Null),
        }
    }
}

impl ToValue for crate::model::Value {
    fn to_value(&self) -> Result<Value, Error> {
        use crate::model::Value as ModelValue;
        use std::ops::Deref;
        match self {
            ModelValue::Integer(i) => i.to_value(),
            ModelValue::Float(f) => f.to_value(),
            ModelValue::String(s) => s.to_value(),
            ModelValue::Optional(v) => match v.deref() {
                Some(v) => v.to_value(),
                None => Ok(Value::Null),
            },
            value => todo!("model value {:?} to template value", value),
        }
    }
}

impl<T: Model> ToValue for T {
    fn to_value(&self) -> Result<Value, Error> {
        let columns = T::column_names();
        let values = self.values();

        if columns.len() != values.len() {
            return Err(Error::SerializationError);
        }

        let mut hash = HashMap::from([("id".to_string(), self.id().to_value()?)]);

        for (key, value) in columns.iter().zip(values.iter()) {
            hash.insert(key.to_string(), value.to_value()?);
        }

        Ok(Value::Hash(hash))
    }
}

impl<T: Model> ToValue for Vec<T> {
    fn to_value(&self) -> Result<Value, Error> {
        let mut list = vec![];
        for v in self.iter() {
            list.push(v.to_value()?);
        }
        Ok(Value::List(list))
    }
}

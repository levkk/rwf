use super::Error;

/// A constant value, e.g. `5` or `"hello world"`.
#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Null,
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
            Value::Null => write!(f, "null"),
        }
    }
}

impl Value {
    /// If the value, when evaluated in the context of a `if` statement expression
    /// would result in the `if` statement being executed.
    ///
    /// e.g. `<% if 5 %>five is true<% end %>`
    /// would output "five is true" since `5` is truthy.
    pub fn truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            _ => true,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        match (self, other) {
            (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 + i2),
            (Value::Integer(i1), Value::Float(f2)) => Value::Float(*i1 as f64 + f2),
            (Value::Float(f1), Value::Integer(i2)) => Value::Float(f1 + *i2 as f64),
            (Value::Float(f1), Value::Float(f2)) => Value::Float(f1 + f2),
            (Value::String(s1), Value::String(s2)) => Value::String(format!("{}{}", s1, s2)),
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

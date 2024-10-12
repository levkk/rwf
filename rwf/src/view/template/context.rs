use crate::view::template::{Error, ToValue, Value};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

#[derive(Debug, Default, Clone)]
pub struct Context {
    values: HashMap<String, Value>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.values.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: impl ToValue) -> Result<&mut Self, Error> {
        self.values.insert(key.to_string(), value.to_value()?);
        Ok(self)
    }
}

impl TryFrom<HashMap<String, Value>> for Context {
    type Error = Error;

    fn try_from(values: HashMap<String, Value>) -> Result<Context, Self::Error> {
        Ok(Context { values })
    }
}

impl TryFrom<&Context> for Context {
    type Error = Error;

    fn try_from(context: &Context) -> Result<Context, Self::Error> {
        Ok(context.clone())
    }
}

macro_rules! impl_string {
    ($ty:ty) => {
       impl TryFrom<$ty> for Context {
            type Error = Error;

            fn try_from(values: $ty) -> Result<Context, Self::Error> {
                let mut result = HashMap::<String, Value>::new();
                for (key, value) in values {
                    result.insert(key.to_string(), Value::String(value.to_string()));
                }

                Ok(Context { values: result })
            }
        }
    }
}

macro_rules! impl_integer {
    ($ty:ty) => {
       impl TryFrom<$ty> for Context {
            type Error = Error;

            fn try_from(values: $ty) -> Result<Context, Self::Error> {
                let mut result = HashMap::<String, Value>::new();
                for (key, value) in values {
                    result.insert(key.to_string(), Value::Integer(value as i64));
                }

                Ok(Context { values: result })
            }
        }
    }
}

macro_rules! impl_impl_integer {
    ($ty:ty) => {
        impl_integer!(HashMap<String, $ty>);
        impl_integer!(HashMap<&str, $ty>);
        impl_integer!(Vec<(&str, $ty)>);
        impl_integer!([(&str, $ty); 1]);
        impl_integer!([(&str, $ty); 2]);
        impl_integer!([(&str, $ty); 3]);
        impl_integer!([(&str, $ty); 4]);
        impl_integer!([(&str, $ty); 5]);
        impl_integer!([(&str, $ty); 6]);
        impl_integer!([(&str, $ty); 7]);
        impl_integer!([(&str, $ty); 8]);
        impl_integer!([(&str, $ty); 9]);
        impl_integer!([(&str, $ty); 10]);
        impl_integer!([(&str, $ty); 11]);
        impl_integer!([(&str, $ty); 12]);
    }
}

impl_string!(HashMap<String, String>);
impl_string!(HashMap<&str, &str>);
impl_string!(Vec<(&str, &str)>);
impl_string!([(&str, &str); 1]);
impl_string!([(&str, &str); 2]);
impl_string!([(&str, &str); 3]);
impl_string!([(&str, &str); 4]);
impl_string!([(&str, &str); 5]);
impl_string!([(&str, &str); 6]);
impl_string!([(&str, &str); 7]);
impl_string!([(&str, &str); 8]);
impl_string!([(&str, &str); 9]);
impl_string!([(&str, &str); 10]);
impl_string!([(&str, &str); 11]);
impl_string!([(&str, &str); 12]);

impl_string!([(&str, String); 1]);
impl_string!([(&str, String); 2]);
impl_string!([(&str, String); 3]);
impl_string!([(&str, String); 4]);
impl_string!([(&str, String); 5]);
impl_string!([(&str, String); 6]);
impl_string!([(&str, String); 7]);
impl_string!([(&str, String); 8]);
impl_string!([(&str, String); 9]);
impl_string!([(&str, String); 10]);
impl_string!([(&str, String); 11]);
impl_string!([(&str, String); 12]);

impl_impl_integer!(i64);
impl_impl_integer!(i32);
impl_impl_integer!(i16);
impl_impl_integer!(i8);

impl_impl_integer!(u64);
impl_impl_integer!(u32);
impl_impl_integer!(u16);
impl_impl_integer!(u8);


// impl TryFrom<HashMap<String, String>> for Context {
//     type Error = Error;

//     fn try_from(values: HashMap<String, String>) -> Result<Context, Self::Error> {
//         let mut result = HashMap::<String, Value>::new();
//         for (key, value) in values {
//             result.insert(key, Value::String(value));
//         }

//         Ok(Context { values: result })
//     }
// }

// impl TryFrom<HashMap<&str, &str>> for Context {
//     type Error = Error;

//     fn try_from(values: HashMap<&str, &str>) -> Result<Context, Self::Error> {
//         let mut result = HashMap::<String, Value>::new();
//         for (key, value) in values {
//             result.insert(key.to_string(), Value::String(value.to_string()));
//         }

//         Ok(Context { values: result })
//     }
// }

// impl TryFrom<Vec<(&str, &str)>> for Context {
//     type Error = Error;

//     fn try_from(values: Vec<(&str, &str)>) -> Result<Context, Self::Error> {
//         let mut result = HashMap::<String, Value>::new();
//         for (key, value) in values {
//             result.insert(key.to_string(), Value::String(value.to_string()));
//         }

//         Ok(Context { values: result })
//     }
// }

impl Index<&str> for Context {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        self.values.get(key).unwrap_or(&Value::Null)
    }
}

impl IndexMut<&str> for Context {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        if let Some(_value) = self.values.get(key) {
            self.values.get_mut(key).unwrap()
        } else {
            self.values.insert(key.to_string(), Value::Null);
            self.values.get_mut(key).unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_context_index() {
        let mut context = Context::default();
        context["test"] = "value".to_value().expect("to_value");

        assert_eq!(context["test"], Value::String("value".to_string()));
    }
}

use crate::view::template::{Error, ToTemplateValue, Value};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::sync::Arc;

use once_cell::sync::Lazy;

static DEFAULTS: Lazy<Arc<RwLock<Context>>> =
    Lazy::new(|| Arc::new(RwLock::new(Context::default())));

#[derive(Debug, Default, Clone)]
pub struct Context {
    values: HashMap<String, Value>,
}

impl Context {
    pub fn new() -> Self {
        DEFAULTS.read().clone()
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.values.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: impl ToTemplateValue) -> Result<&mut Self, Error> {
        self.values
            .insert(key.to_string(), value.to_template_value()?);
        Ok(self)
    }

    pub fn defaults(context: Self) {
        (*DEFAULTS.write()) = context;
    }
}

impl ToTemplateValue for Context {
    fn to_template_value(&self) -> Result<Value, Error> {
        Ok(Value::Hash(self.values.clone()))
    }
}

impl TryFrom<&Context> for Context {
    type Error = Error;

    fn try_from(context: &Context) -> Result<Context, Self::Error> {
        Ok(context.clone())
    }
}

macro_rules! impl_type {
    ($ty:ty) => {
        impl TryFrom<$ty> for Context {
            type Error = Error;

            fn try_from(values: $ty) -> Result<Context, Self::Error> {
                let mut result = HashMap::<String, Value>::new();
                for (key, value) in values {
                    result.insert(key.to_string(), value.to_template_value()?);
                }

                Ok(Context { values: result })
            }
        }
    };
}

macro_rules! impl_impl_type {
    ($ty:ty) => {
        impl_type!(HashMap<String, $ty>);
        impl_type!(HashMap<&str, $ty>);
        impl_type!(Vec<(&str, $ty)>);
        impl_type!([(&str, $ty); 1]);
        impl_type!([(&str, $ty); 2]);
        impl_type!([(&str, $ty); 3]);
        impl_type!([(&str, $ty); 4]);
        impl_type!([(&str, $ty); 5]);
        impl_type!([(&str, $ty); 6]);
        impl_type!([(&str, $ty); 7]);
        impl_type!([(&str, $ty); 8]);
        impl_type!([(&str, $ty); 9]);
        impl_type!([(&str, $ty); 10]);
        impl_type!([(&str, $ty); 11]);
        impl_type!([(&str, $ty); 12]);
    }
}

impl_impl_type!(i64);
impl_impl_type!(i32);
impl_impl_type!(i16);
impl_impl_type!(i8);
impl_impl_type!(u64);
impl_impl_type!(u32);
impl_impl_type!(u16);
impl_impl_type!(u8);
impl_impl_type!(Value);
impl_impl_type!(String);
impl_impl_type!(&str);
impl_impl_type!(f32);
impl_impl_type!(f64);
impl_impl_type!(time::OffsetDateTime);

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
        context["test"] = "value".to_template_value().expect("to_template_value");

        assert_eq!(context["test"], Value::String("value".to_string()));
    }
}

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

impl From<HashMap<String, Value>> for Context {
    fn from(values: HashMap<String, Value>) -> Context {
        Context { values }
    }
}

impl Index<&str> for Context {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        self.values.get(key).unwrap_or(&Value::Null)
    }
}

impl IndexMut<&str> for Context {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        if let Some(value) = self.values.get(key) {
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

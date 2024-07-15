use crate::view::template::Value;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Context {}

impl Context {
    pub fn get(&self, key: &str) -> Option<Value> {
        todo!("get {}", key)
    }
}

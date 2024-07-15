use std::collections::HashMap;

use super::super::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    String(String),
    Integer(i64),
    Float(f64),
    List(Vec<Constant>),
    Hash(HashMap<String, Constant>),
}

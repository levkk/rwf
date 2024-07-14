use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    String(String),
    Integer(i64),
    Float(f64),
    List(Vec<Constant>),
    Hash(HashMap<String, Constant>),
}

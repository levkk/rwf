use std::collections::HashMap;

#[derive(Debug)]
pub enum Context {
    Hash(HashMap<String, Context>),
    ValueString(String),
    ValueInteger(i64),
}

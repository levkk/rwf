use super::Value;

#[derive(Debug, Clone, Default)]
pub struct Placeholders {
    values: Vec<Value>,
}

impl Placeholders {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn add(&mut self, value: &Value) -> Value {
        let id = self.values.len() + 1;
        self.values.push(value.clone());
        Value::Placeholder(id as i32)
    }

    pub fn values(&self) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
        self.values
            .iter()
            .map(|v| v as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect()
    }
}

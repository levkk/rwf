use super::ToSql;

#[derive(Debug, Default)]
pub struct Lock {
    lock: bool,
}

impl Lock {
    pub fn new() -> Self {
        Self { lock: true }
    }
}

impl ToSql for Lock {
    fn to_sql(&self) -> String {
        if self.lock { " FOR UPDATE" } else { "" }.into()
    }
}

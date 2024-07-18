use super::ToSql;

#[derive(Debug, Default)]
pub struct Lock {
    lock: bool,
    skip_locked: bool,
}

impl Lock {
    pub fn new() -> Self {
        Self {
            lock: true,
            skip_locked: false,
        }
    }

    pub fn skip_locked(mut self) -> Self {
        self.skip_locked = true;
        self
    }
}

impl ToSql for Lock {
    fn to_sql(&self) -> String {
        let lock = if self.lock { " FOR UPDATE" } else { "" };
        let skip_locked = if self.skip_locked { " SKIP LOCKED" } else { "" };
        format!("{}{}", lock, skip_locked)
    }
}

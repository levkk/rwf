//! Select only a few columns.
#![allow(dead_code, unused_variables)]

use std::collections::HashMap;

use super::*;

#[derive(Debug, Clone)]
pub struct Picked<T: FromRow> {
    pub select: Select<T>,
    columns: HashMap<Column, Value>,
}

impl<T: FromRow> Picked<T> {
    pub fn group(mut self, group: &[impl ToColumn]) -> Self {
        self.select = self.select.group(group);
        self
    }

    fn from_row(&self, row: tokio_postgres::Row) -> Result<Self, Error> {
        todo!()
    }
}

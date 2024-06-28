use crate::model::{Column, Columns, Limit, OrderBy, ToSql, Value, Where};

#[derive(Debug)]
pub struct Select {
    pub table_name: String,
    pub columns: Columns,
    pub where_: Where,
    pub order_by: OrderBy,
    pub limit: Limit,
}

impl Select {
    pub fn limit(mut self, limit: Limit) -> Self {
        self.limit = limit;
        self
    }

    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by = order_by;
        self
    }

    pub fn where_mut(&mut self) -> &mut Where {
        &mut self.where_
    }
}

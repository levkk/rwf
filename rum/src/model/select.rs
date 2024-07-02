use crate::model::{
    filter::{Filter, JoinOp},
    Column, Columns, Limit, OrderBy, Placeholders, ToValue, Value, WhereClause,
};

#[derive(Debug, Default)]
pub struct Select {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Columns,
    pub order_by: OrderBy,
    pub limit: Limit,
    pub placeholders: Placeholders,
    pub where_clause: WhereClause,
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

    pub fn where_clause_mut(&mut self) -> &mut WhereClause {
        &mut self.where_clause
    }

    pub fn placeholders_mut(&mut self) -> &mut Placeholders {
        &mut self.placeholders
    }

    pub fn filter(mut self, filters: &[(impl ToString, impl ToValue)], join_op: JoinOp) -> Self {
        let mut filter = Filter::default();
        let table_name = self.table_name.clone();

        for (column, value) in filters.into_iter() {
            let column = Column::new(&table_name, &column.to_string().as_str());
            let value = value.to_value();

            let value = match value {
                Value::List(_) => {
                    let placeholder = self.placeholders_mut().add(&value);
                    Value::Record(Box::new(placeholder))
                }

                value => self.placeholders_mut().add(&value),
            };

            filter.add(column, value);
        }

        match join_op {
            JoinOp::And => self.where_clause.concat(filter),
            JoinOp::Or => self.where_clause.or(filter),
        };

        self
    }

    pub fn filter_and(mut self, filters: &[(impl ToString, impl ToValue)]) -> Self {
        self = self.filter(filters, JoinOp::And);
        self
    }

    pub fn filter_or(mut self, filters: &[(impl ToString, impl ToValue)]) -> Self {
        self = self.filter(filters, JoinOp::Or);
        self
    }
}

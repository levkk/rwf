use super::{Column, Escape, ToSql};

#[derive(Debug, Clone)]
pub enum OrderColumn {
    Asc(Column),
    Desc(Column),
    Raw(String),
}

impl ToSql for OrderColumn {
    fn to_sql(&self) -> String {
        use OrderColumn::*;

        match self {
            Asc(column) => format!("{} ASC", column.to_sql()),
            Desc(column) => format!("{} DESC", column.to_sql()),
            Raw(raw) => raw.clone(),
        }
    }
}

pub trait ToOrderBy {
    fn to_order_by(&self) -> OrderBy;
}

impl ToOrderBy for &str {
    fn to_order_by(&self) -> OrderBy {
        OrderBy {
            order_by: vec![OrderColumn::Raw(self.escape())],
        }
    }
}

impl ToOrderBy for [&str; 2] {
    fn to_order_by(&self) -> OrderBy {
        OrderBy {
            order_by: vec![OrderColumn::Raw(format!(
                "{} {}",
                Column::name(self[0]).to_sql(),
                self[1].to_ascii_uppercase().escape()
            ))],
        }
    }
}

impl ToOrderBy for (&str, &str) {
    fn to_order_by(&self) -> OrderBy {
        [self.0, self.1].to_order_by()
    }
}

#[derive(Debug, Default, Clone)]
pub struct OrderBy {
    pub order_by: Vec<OrderColumn>,
}

impl OrderBy {
    pub fn asc(column: Column) -> Self {
        Self {
            order_by: vec![OrderColumn::Asc(column)],
        }
    }

    pub fn desc(column: Column) -> Self {
        Self {
            order_by: vec![OrderColumn::Desc(column)],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.order_by.is_empty()
    }
}

impl std::ops::Add for OrderBy {
    type Output = OrderBy;

    fn add(mut self, other: OrderBy) -> Self {
        self.order_by.extend(other.order_by);
        self
    }
}

impl ToSql for OrderBy {
    fn to_sql(&self) -> String {
        if self.order_by.is_empty() {
            "".to_string()
        } else {
            format!(
                " ORDER BY {}",
                self.order_by
                    .iter()
                    .map(|column| column.to_sql())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_order_by_raw() {
        let _order_by = "created_at ASC".to_order_by();
        let _order_by = ["created_at", "ASC"].to_order_by();
    }
}

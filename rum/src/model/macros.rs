#[macro_export]
macro_rules! belongs_to {
    ($a:ident, $b:ident) => {
        impl crate::model::Association<$a> for $b {}
    };
}

#[macro_export]
macro_rules! has_many {
    ($a:ident, $b:ident) => {
        impl crate::model::Association<$a> for $b {
            fn association_type() -> crate::model::AssociationType {
                crate::model::AssociationType::HasMany
            }
        }
    };
}

pub use belongs_to;
pub use has_many;

#[cfg(test)]
mod test {
    use super::super::{FromRow, Model};
    use super::*;

    #[derive(Clone, Default)]
    struct User {}

    #[derive(Clone, Default, rum_macros::Model)]
    struct Order {}

    impl Model for User {}
    impl Model for Order {}

    impl FromRow for User {
        fn from_row(row: tokio_postgres::Row) -> Self {
            Self::default()
        }
    }

    impl FromRow for Order {
        fn from_row(row: tokio_postgres::Row) -> Self {
            Self::default()
        }
    }

    belongs_to!(Order, User);
    has_many!(User, Order);

    #[test]
    fn test_macros() {
        assert_eq!(User::foreign_key(), "user_id");
        assert_eq!(Order::foreign_key(), "order_id");
        assert_eq!(User::table_name(), "users");
        assert_eq!(Order::table_name(), "orders");
    }
}

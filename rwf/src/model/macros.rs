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

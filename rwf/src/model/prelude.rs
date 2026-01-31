//! Include all types in here to use the ORM ergonomically.
//!
//! These types are covered by [`crate::prelude`].
pub use self::{datatypes::*, traits::*};
pub mod traits {
    pub use crate::model::cursor::{
        Cursor, DecoupleTransactionCursor, FetchableCursor, TargetedCursor, ToTransactionCursor,
        TransactionCursor,
    };
    pub use crate::model::{
        combine::CombinedQuery, select::FilterQuery, temporary::WithQuery, FromRow, Model,
        ToColumn, ToSql, ToValue,
    };
}
pub mod datatypes {
    pub use crate::model::cursor::{
        DeclareCursor, ModelCursor, SelectiveCursor, TxModelCursor, TxSelectiveCursor,
    };
    pub use crate::model::{Column, Error, Pool, Query, Scope, Value};
}

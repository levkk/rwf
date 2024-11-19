//! Implementation of the template language.
//!
//! Includes the parser and runtime.
pub mod expression;
pub mod op;
pub mod program;
pub mod statement;
pub mod term;

pub use expression::Expression;
pub use op::Op;
pub use program::Program;
pub use statement::Statement;
pub use term::Term;

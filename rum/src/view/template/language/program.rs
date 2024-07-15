use super::super::{Error, TokenWithLine};
use super::Statement;

pub struct Program {
    statements: Vec<Statement>,
}

impl Program {
    pub fn parse(tokens: Vec<TokenWithLine>) -> Result<Self, Error> {
        todo!()
    }
}

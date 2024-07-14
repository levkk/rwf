use super::{
    super::{Error, Token, TokenWithLine},
    Constant, Expression, Term,
};
use std::iter::{Iterator, Peekable};

pub enum Statement {
    Print(Expression),
    If {
        expression: Expression,
        if_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },

    For {
        variable: Term,
        list: Constant,
        body: Vec<Statement>,
    },

    Nothing,
}

impl Statement {
    pub fn parse(
        iter: &mut Peekable<impl Iterator<Item = TokenWithLine>>,
    ) -> Result<Statement, Error> {
        todo!()
    }
}

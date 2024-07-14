use super::super::tokenizer::{Comparison, Token};
use crate::model::error::Error;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Op {
    Not,
    And,
    Or,
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    Eq,
    Neq,
}

impl From<Token> for Option<Op> {
    fn from(token: Token) -> Option<Op> {
        Some(match token {
            Token::Not => Op::Not,
            Token::And => Op::And,
            Token::Or => Op::Or,
            Token::Comparison(Comparison::Equals) => Op::Eq,
            _ => return None,
        })
    }
}

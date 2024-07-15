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
    Equals,
    NotEquals,
    GreaterThan,
    GreaterEqualThan,
    LessThan,
    LessEqualThan,
}

impl Op {
    pub fn from_token(token: Token) -> Option<Self> {
        Option::<Self>::from(token)
    }

    pub fn binary(&self) -> bool {
        match self {
            Op::Not => false,
            _ => true,
        }
    }
}

impl From<Token> for Option<Op> {
    fn from(token: Token) -> Option<Op> {
        Some(match token {
            Token::Not => Op::Not,
            Token::And => Op::And,
            Token::Or => Op::Or,
            Token::Equals => Op::Equals,
            Token::NotEquals => Op::NotEquals,
            Token::GreaterThan => Op::GreaterThan,
            Token::GreaterEqualThan => Op::GreaterEqualThan,
            Token::LessThan => Op::LessThan,
            Token::LessEqualThan => Op::LessEqualThan,
            _ => return None,
        })
    }
}

use super::super::lexer::{Token, Value};
use super::super::Error;

use std::cmp::Ordering;

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

impl PartialOrd for Op {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_prec = self.precendence();
        let other_prec = other.precendence();
        self_prec.partial_cmp(&other_prec)
    }
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

    pub fn evaluate_binary(&self, left: &Value, right: &Value) -> Result<Value, Error> {
        match self {
            Op::Equals => Ok(Value::Boolean(left == right)),
            Op::NotEquals => Ok(Value::Boolean(left != right)),
            Op::LessThan => Ok(Value::Boolean(left < right)),
            Op::LessEqualThan => Ok(Value::Boolean(left <= right)),
            Op::GreaterThan => Ok(Value::Boolean(left > right)),
            Op::GreaterEqualThan => Ok(Value::Boolean(left >= right)),
            Op::And => Ok(Value::Boolean(left.truthy() && right.truthy())),
            Op::Or => Ok(Value::Boolean(left.truthy() || right.truthy())),
            _ => todo!(),
        }
    }

    // Source: <https://en.cppreference.com/w/c/language/operator_precedence>
    pub fn precendence(&self) -> u8 {
        match self {
            Op::Not => 1,
            Op::And => 11,
            Op::Or => 12,
            Op::Add | Op::Sub => 4,
            Op::Mult | Op::Div | Op::Mod => 3,
            Op::Equals | Op::NotEquals => 7,
            _ => todo!(),
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
            Token::Plus => Op::Add,
            Token::Minus => Op::Sub,
            Token::Mult => Op::Mult,
            Token::Div => Op::Div,
            _ => return None,
        })
    }
}

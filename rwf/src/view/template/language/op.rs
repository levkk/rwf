//! Mathematical operation between data types.
use super::super::lexer::{Token, Value};
use super::super::Error;

use std::cmp::Ordering;

/// List of supported operations, e.g. addition, equality, etc.
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
    /// Convert a language token to an op. If the token
    /// isn't an op, `None` is returned.
    pub fn from_token(token: Token) -> Option<Self> {
        Option::<Self>::from(token)
    }

    /// Is this a binary operator, i.e. an operation between two terms?
    pub fn binary(&self) -> bool {
        match self {
            Op::Not => false,
            _ => true,
        }
    }

    /// Evaluate the operation on a value.
    pub fn evaluate_unary(&self, value: &Value) -> Result<Value, Error> {
        match self {
            Op::Not => Ok(Value::Boolean(!value.truthy())),
            Op::Sub => Ok(match value {
                Value::Integer(integer) => Value::Integer(-integer),
                Value::Float(float) => Value::Float(-float),
                _ => Value::Null,
            }),
            Op::Add => Ok(value.clone()),
            _ => Ok(Value::Null),
        }
    }

    /// Combinate two terms into one using the operation.
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
            Op::Add => Ok(left.add(right)),
            Op::Sub => Ok(left.sub(right)),
            Op::Mult => Ok(left.mul(right)),
            Op::Div => Ok(left.div(right)),
            _ => todo!(),
        }
    }

    /// Calculate operator precendence, i.e. in an expression with multiple
    /// operations, determine their order of execution.
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

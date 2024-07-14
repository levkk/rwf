use super::{super::tokenizer::Token, Constant};
use crate::model::error::Error;

#[derive(Debug, Clone)]
pub enum Term {
    Constant(Constant),
    Variable(String),
    Function(fn() -> String),
}

impl From<Token> for Option<Term> {
    fn from(token: Token) -> Option<Term> {
        Some(match token {
            Token::Variable(name) => Term::Variable(name),
            Token::Integer(integer) => Term::Constant(Constant::Integer(integer)),
            Token::Float(float) => Term::Constant(Constant::Float(float)),
            Token::String(string) => Term::Constant(Constant::String(string)),
            _ => return None,
        })
    }
}

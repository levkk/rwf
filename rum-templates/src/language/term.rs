//! Term, basic building block of the template language.
//!
//! A term can be a constant or a variable.
use super::super::{
    lexer::{Token, Value},
    Context,
};

use crate::Error;

// Term parser and executor.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Constant(Value),
    Variable(String),
}

impl Term {
    /// Create a term from a token.
    pub fn from_token(token: Token) -> Option<Self> {
        Option::<Self>::from(token)
    }

    /// Create a constant from a value.
    pub fn constant(value: Value) -> Self {
        Term::Constant(value)
    }

    /// Create a variable from a name.
    pub fn variable(name: String) -> Self {
        Term::Variable(name)
    }

    /// Evaluate the term given the context.
    pub fn evaluate(&self, context: &Context) -> Result<Value, Error> {
        match self {
            Term::Constant(value) => Ok(value.clone()),
            Term::Variable(name) => context
                .get(&name)
                .ok_or(Error::UndefinedVariable(name.clone())),
        }
    }
}

impl From<Token> for Option<Term> {
    fn from(token: Token) -> Option<Term> {
        Some(match token {
            Token::Variable(name) => Term::Variable(name),
            Token::Value(value) => Term::Constant(value),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Lexer;
    use std::collections::HashMap;

    #[test]
    fn test_terms() -> Result<(), Error> {
        let t1 = "<% 1 %>";
        let tokens = Lexer::new(&t1).tokens()?;
        let integer = Term::from_token(tokens[1].token());
        assert_eq!(
            integer.expect("integer").evaluate(&Context::default())?,
            Value::Integer(1)
        );

        let t2 = r#"<% "string" %>"#;
        let tokens = Lexer::new(&t2).tokens()?;
        let string = Term::from_token(tokens[1].token());
        assert_eq!(
            string.expect("string").evaluate(&Context::default())?,
            Value::String("string".into())
        );

        let t3 = "<% 1.54 %>";
        let tokens = Lexer::new(&t3).tokens()?;
        let float = Term::from_token(tokens[1].token());
        assert_eq!(
            float.expect("float").evaluate(&Context::default())?,
            Value::Float(1.54)
        );

        let t4 = "<% variable %>";
        let tokens = Lexer::new(&t4).tokens()?;
        let variable = Term::from_token(tokens[1].token());
        let _value = variable
            .expect("variable")
            .evaluate(&Context::from(HashMap::from([(
                "variable".to_string(),
                Value::String("test".into()),
            )])));

        Ok(())
    }
}

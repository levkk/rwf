use super::{
    super::{
        lexer::{Token, Value},
        Context,
    },
    Expression,
};
use crate::view::template::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Constant(Value),
    Variable(String),
    List(Vec<Value>),
    Function(fn() -> String),
}

impl Term {
    pub fn from_token(token: Token) -> Option<Self> {
        Option::<Self>::from(token)
    }

    pub fn constant(value: Value) -> Self {
        Term::Constant(value)
    }

    pub fn variable(name: String) -> Self {
        Term::Variable(name)
    }

    pub fn evaluate(&self, context: &Context) -> Result<Value, Error> {
        match self {
            Term::Constant(value) => Ok(value.clone()),
            Term::List(values) => Ok(Value::List(values.clone())),
            Term::Variable(name) => context
                .get(&name)
                .ok_or(Error::UndefinedVariable(name.clone())),
            _ => todo!(),
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
    use crate::view::template::Lexer;
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
        let value = variable
            .expect("variable")
            .evaluate(&Context::new(HashMap::from([(
                "variable".to_string(),
                Value::String("test".into()),
            )])));

        Ok(())
    }
}

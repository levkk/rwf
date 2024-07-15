use super::{
    super::tokenizer::{Token, Value},
    Constant,
};
use crate::view::template::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Constant(Value),
    Variable(String),
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
    use crate::view::template::Tokenizer;

    #[test]
    fn test_terms() -> Result<(), Error> {
        let t1 = "<% 1 %>";
        let tokens = Tokenizer::new(&t1).tokens()?;
        let integer = Term::from_token(tokens[2].token());
        assert_eq!(integer, Some(Term::Constant(Value::Integer(1))));

        let t2 = r#"<% "string" %>"#;
        let tokens = Tokenizer::new(&t2).tokens()?;
        let string = Term::from_token(tokens[2].token());
        assert_eq!(string, Some(Term::Constant(Value::String("string".into()))));

        let t3 = "<% 1.54 %>";
        let tokens = Tokenizer::new(&t3).tokens()?;
        let float = Term::from_token(tokens[2].token());
        assert_eq!(float, Some(Term::Constant(Value::Float(1.54))));

        let t4 = "<% variable %>";
        let tokens = Tokenizer::new(&t4).tokens()?;
        let variable = Term::from_token(tokens[2].token());
        assert_eq!(variable, Some(Term::Variable("variable".into())));

        Ok(())
    }
}

use super::tokenizer::{Comparison, Token};
use crate::model::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Expression(Vec<Language>),
    Text(String),
    Print(Expression),
    Variable(String),
    Comparison(Box<Language>, Comparison, Box<Language>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    code_block: bool,
    buffer: Vec<Language>,
}

macro_rules! next_token {
    ($iter:expr) => {
        match $iter.next() {
            Some(token) => token,
            None => return Err(Error::SyntaxError("expected token, got eof".into())),
        }
    };
}

macro_rules! consume_spaces {
    ($iter:expr) => {
        match $iter.next() {
            Some(Token::Space) => continue,
            Some(token) => token,
            None => return Err(Error::SyntaxError("expected token, got eof".into())),
        }
    };
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        Self {
            tokens: tokens.to_vec(),
            code_block: false,
            buffer: vec![],
        }
    }

    pub fn eval(mut self) -> Result<Vec<Language>, Error> {
        let mut iter = self.tokens.into_iter();

        while let Some(token) = iter.next() {
            match token {
                Token::Text(string) => {
                    self.buffer.push(Language::Text(string));
                }

                Token::If => {
                    let next = consume_spaces!(iter);

                    match next {
                        Token::Comparison(comparison) => {
                            let next = consume_spaces!(iter);

                            match next {
                                Token::Variable(variable) => {}
                                Token::Integer(integer) => {}
                                Token::Float(float) => {}
                                Token::String(string) => {}
                                _ => (),
                            }
                        }

                        Token::BlockEnd => {
                            // Eval that the variable exists in the context
                        }

                        _ => {
                            return Err(Error::SyntaxError(format!(
                                "expected Comparison, got {:?}",
                                next
                            )))
                        }
                    }
                }

                Token::Text(string) => {
                    self.buffer.push(Language::Text(string));
                }

                _ => {}
            }
        }

        Ok(self.buffer)
    }
}

#[cfg(test)]
mod test {
    use super::super::Tokenizer;
    use super::*;

    #[test]
    fn test_parser() -> Result<(), Error> {
        let template = "<html><% if hello == 1 %><hello></hello><% else %><world></world><% end %>";
        let tokens = Tokenizer::new(template).tokens()?;
        println!("{:?}", tokens);
        let language = Parser::new(&tokens).eval()?;
        println!("{:?}", language);

        Ok(())
    }
}

use super::tokenizer::{Comparison, Token, TokenWithLine};
use crate::model::Error;
use std::iter::Iterator;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    // Expression(Expression),
    Text(String),
    // Print(Expression),
    Variable(String),
    Comparison(Box<Language>, Comparison, Box<Language>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Binary(Comparison, Operand, Operand),
    Unary(Comparison, Operand),
}

#[derive(Debug)]
pub enum Expression {
    If(If),
    Print(String),
}

#[derive(Debug)]
pub struct If {
    condition: Operation,
    if_body: Vec<Expression>,
    else_body: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Variable(String),
    Value(Value),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<TokenWithLine>,
    code_block: bool,
    buffer: Vec<Expression>,
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
        loop {
            match $iter.next() {
                Some(TokenWithLine {
                    token: Token::Space,
                    ..
                }) => continue,
                Some(token) => {
                    break token;
                }
                None => return Err(Error::SyntaxError("expected token, got eof".into())),
            }
        }
    };
}

macro_rules! syntax_error {
    ($message:expr) => {
        return Err(Error::SyntaxError($message.into()));
    };
}

impl Parser {
    pub fn new(tokens: &[TokenWithLine]) -> Self {
        Self {
            tokens: tokens.to_vec(),
            code_block: false,
            buffer: vec![],
        }
    }

    pub fn eval(mut self) -> Result<Vec<Expression>, Error> {
        let mut iter = self.tokens.into_iter();

        while let Some(token) = iter.next() {
            match token.token() {
                Token::Text(string) => {
                    self.buffer.push(Expression::Print(string));
                }

                Token::If => {
                    let expression = Self::parse_if(&mut iter)?;
                    self.buffer.push(expression);
                }

                Token::BlockStart => (),
                Token::Space => (),
                Token::EndIf => (),
                Token::BlockEnd => (),

                token => syntax_error!(format!("unexpected token {:?}", token)),
            }
        }

        Ok(self.buffer)
    }

    fn parse_if(iter: &mut impl Iterator<Item = TokenWithLine>) -> Result<Expression, Error> {
        let next = consume_spaces!(iter);

        match next.token() {
            Token::Variable(variable) => {
                let next = consume_spaces!(iter);

                let (comparison, operand) = match next.token() {
                    Token::Comparison(comparison) => {
                        let next = consume_spaces!(iter);

                        let op = match next.token() {
                            Token::Variable(variable) => (comparison, Operand::Variable(variable)),
                            Token::Integer(integer) => {
                                (comparison, Operand::Value(Value::Integer(integer)))
                            }
                            Token::Float(float) => {
                                (comparison, Operand::Value(Value::Float(float)))
                            }
                            Token::String(string) => {
                                (comparison, Operand::Value(Value::String(string)))
                            }
                            _ => syntax_error!("expected variable or value"),
                        };

                        let next = consume_spaces!(iter);

                        if next.token() != Token::BlockEnd {
                            syntax_error!("expected block end");
                        }

                        op
                    }

                    Token::BlockEnd => (Comparison::True, Operand::None),

                    _ => syntax_error!("expected comparison or block end"),
                };

                let condition = match operand {
                    Operand::None => Operation::Unary(comparison, Operand::Variable(variable)),
                    operand => Operation::Binary(comparison, Operand::Variable(variable), operand),
                };

                let if_body = Self::parse_body(iter)?;
                let next = consume_spaces!(iter);

                let else_body = match next.token() {
                    Token::ElseIf => vec![Self::parse_if(iter)?],
                    Token::Else => {
                        let next = consume_spaces!(iter);
                        if next.clone().token() != Token::BlockEnd {
                            syntax_error!(format!("expected block end in else, got {:?}", next));
                        }

                        Self::parse_body(iter)?
                    }
                    Token::EndIf => vec![],
                    _ => syntax_error!("expected else if, else, end if"),
                };

                Ok(Expression::If(If {
                    condition,
                    if_body,
                    else_body,
                }))
            }

            _ => syntax_error!("expected variable or value"),
        }
    }

    fn parse_body(
        iter: &mut impl Iterator<Item = TokenWithLine>,
    ) -> Result<Vec<Expression>, Error> {
        let mut body = vec![];

        loop {
            let next = consume_spaces!(iter);

            match next.token() {
                Token::Text(text) => body.push(Expression::Print(text)),
                Token::BlockStart => break,
                token => syntax_error!(format!("expected print or another block, got {:?}", token)),
            }
        }

        Ok(body)
    }
}

#[cfg(test)]
mod test {
    use super::super::Tokenizer;
    use super::*;

    #[test]
    fn test_parser() -> Result<(), Error> {
        let template = "<html>
                <% if hello == 1 %>
                    <hello></hello>
                <% elsif hello == 2 %>
                    <world></world>
                <% else %>
                    Other
                <% end %>
            </html>";
        let tokens = Tokenizer::new(template).tokens()?;
        let language = Parser::new(&tokens).eval()?;
        println!("{:#?}", language);

        Ok(())
    }
}

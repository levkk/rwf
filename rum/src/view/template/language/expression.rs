use super::{
    super::tokenizer::{Comparison, Token, TokenWithLine, Value},
    super::Error,
    Constant, Op, Term,
};

use std::iter::{Iterator, Peekable};

#[derive(Debug, Clone)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        op: Op,
        right: Box<Expression>,
    },

    Unary {
        op: Op,
        operand: Box<Expression>,
    },

    Term {
        term: Term,
    },
}

impl Expression {
    pub fn constant(value: Value) -> Self {
        Self::Term {
            term: Term::constant(value),
        }
    }

    pub fn variable(variable: String) -> Self {
        Self::Term {
            term: Term::variable(variable),
        }
    }

    pub fn evaluate(&self) -> Result<Value, Error> {
        todo!()
    }

    pub fn parse(iter: &mut Peekable<impl Iterator<Item = TokenWithLine>>) -> Result<Self, Error> {
        loop {
            let next = iter.next().ok_or(Error::Eof)?;

            match next.token() {
                Token::BlockStart => (),
                Token::BlockEnd => todo!(),
                Token::Variable(name) => {
                    let left = Self::variable(name);
                    let next = iter.next().ok_or(Error::Eof)?;

                    match Op::from_token(next.token()) {
                        Some(op) => {
                            let right = Expression::parse(iter)?;
                            return Ok(Expression::Binary {
                                left: Box::new(left),
                                op,
                                right: Box::new(right),
                            });
                        }

                        None => return Ok(left),
                    }
                }
                Token::Value(value) => {
                    let left = Self::constant(value);
                    let next = iter.next().ok_or(Error::Eof)?;
                    match Op::from_token(next.token()) {
                        Some(op) => {
                            let right = Expression::parse(iter)?;
                            return Ok(Expression::Binary {
                                left: Box::new(left),
                                op,
                                right: Box::new(right),
                            });
                        }

                        None => return Ok(left),
                    }
                }
                Token::Equals => {
                    let right = Expression::parse(iter)?;
                }
                _ => return Err(Error::Syntax(next)),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::super::tokenizer::Tokenize;
    use super::*;

    #[test]
    fn test_if_const() -> Result<(), Error> {
        let t1 = r#"<% 1 == 2 %>"#.tokenize()?;
        let expr = Expression::parse(&mut t1.into_iter().peekable())?;
        println!("{:?}", expr);

        Ok(())
    }
}

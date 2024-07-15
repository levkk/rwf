use super::{
    super::tokenizer::{Comparison, Token, TokenWithContext, Value},
    super::Context,
    super::Error,
    Op, Term,
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

    pub fn evaluate(&self, context: &Context) -> Result<Value, Error> {
        match self {
            Expression::Term { term } => term.evaluate(context),
            Expression::Binary { left, op, right } => {
                let left = left.evaluate(context)?;
                let right = right.evaluate(context)?;
                op.evaluate_binary(&left, &right)
            }
            _ => todo!(),
        }
    }

    pub fn parse(
        iter: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    ) -> Result<Self, Error> {
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
    use super::super::super::{Context, Tokenize};
    use super::*;

    #[test]
    fn test_if_const() -> Result<(), Error> {
        let t1 = r#"<% 1 == 2 %>"#.tokenize()?;
        let expr = Expression::parse(&mut t1.into_iter().peekable())?;
        let value = expr.evaluate(&Context::default())?;
        assert_eq!(value, Value::Boolean(false));

        let t2 = "<% 1 == 1 %>".tokenize()?;
        let expr = Expression::parse(&mut t2.into_iter().peekable())?;
        let value = expr.evaluate(&Context::default())?;
        assert_eq!(value, Value::Boolean(true));

        Ok(())
    }
}

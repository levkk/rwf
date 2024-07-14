use super::{
    super::tokenizer::{Token, TokenWithLine},
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

    Terms {
        left: Term,
        op: Op,
        right: Term,
    },

    TermExpression {
        left: Term,
        op: Op,
        right: Box<Expression>,
    },

    Term {
        term: Term,
    },

    Print(String),
}

impl Expression {
    pub fn parse(iter: &mut Peekable<impl Iterator<Item = TokenWithLine>>) -> Result<Self, Error> {
        let next = iter.next().ok_or(Error::Eof)?;

        match Option::<Term>::from(next.token.clone()) {
            Some(left_term) => {
                let next = iter.peek();

                match next {
                    Some(token) => {
                        let next = iter.next().ok_or(Error::Eof)?;
                        let op = Option::<Op>::from(next.token.clone());

                        match op {
                            None => return Ok(Expression::Term { term: left_term }),

                            Some(op) => {
                                let next = iter.next().ok_or(Error::Eof)?;
                                match Option::<Term>::from(next.token.clone()) {
                                    Some(right_term) => {
                                        return Ok(Expression::Terms {
                                            left: left_term,
                                            op,
                                            right: right_term,
                                        })
                                    }

                                    None => {
                                        let right_expression = Expression::parse(iter)?;

                                        return Ok(Expression::TermExpression {
                                            left: left_term,
                                            op,
                                            right: Box::new(right_expression),
                                        });
                                    }
                                }
                            }
                        }
                    }

                    None => (),
                }
            }

            None => (),
        };

        let expression = match next.token() {
            Token::Text(text) => return Ok(Expression::Print(text)),
            Token::If => Self::parse(iter),
            _ => todo!(),
        };

        let next = iter.peek().ok_or(Error::Eof)?;

        // if end_block.token() != Token::BlockEnd {
        //     return Err(Error::Eof);
        // }

        // let if_body
        todo!()
    }
}

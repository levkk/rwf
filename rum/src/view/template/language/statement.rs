use super::{
    super::{Context, Error, Token, TokenWithContext, Value},
    Expression, Term,
};
use std::iter::{Iterator, Peekable};

macro_rules! block_end {
    ($iter:expr) => {
        while let Some(token) = $iter.next() {
            match token.token() {
                Token::BlockEnd => break,
                _ => return Err(Error::Syntax(token)),
            }
        }
    };
}

#[derive(Debug)]
pub enum Statement {
    Print(Expression),
    PrintText(String),
    If {
        expression: Expression,
        if_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },

    Else,
    End,

    For {
        variable: Term,
        list: Value,
        body: Vec<Statement>,
    },

    Empty,
}

impl Statement {
    pub fn evaluate(&self, context: &Context) -> Result<String, Error> {
        match self {
            Statement::PrintText(text) => Ok(text.clone()),
            Statement::If {
                expression,
                if_body,
                else_body,
            } => {
                let mut result = String::new();
                if expression.evaluate(&context)?.truthy() {
                    for statement in if_body {
                        result.push_str(&statement.evaluate(&context)?);
                    }
                } else {
                    for statement in else_body {
                        result.push_str(&statement.evaluate(&context)?);
                    }
                }
                Ok(result)
            }
            statement => todo!("evaluating {:?}", statement),
        }
    }

    pub fn parse(
        iter: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    ) -> Result<Statement, Error> {
        loop {
            let next = iter.next().ok_or(Error::Eof)?;
            match next.token() {
                Token::End => {
                    block_end!(iter);
                    return Ok(Statement::End);
                }
                Token::Text(string) => return Ok(Statement::PrintText(string)),
                Token::BlockStart => (),
                Token::Else => {
                    block_end!(iter);
                    return Ok(Statement::Else);
                }
                Token::If | Token::ElseIf => {
                    let (mut if_body, mut else_body) = (vec![], vec![]);
                    let expression = Expression::parse(iter)?;

                    loop {
                        let statement = Statement::parse(iter)?;
                        match statement {
                            Statement::End => {
                                return Ok(Statement::If {
                                    expression,
                                    if_body,
                                    else_body,
                                })
                            }
                            Statement::Else => loop {
                                let statement = Statement::parse(iter)?;

                                match statement {
                                    Statement::End => {
                                        return Ok(Statement::If {
                                            expression,
                                            if_body,
                                            else_body,
                                        })
                                    }
                                    statement => else_body.push(statement),
                                }
                            },
                            statement => if_body.push(statement),
                        }
                    }

                    return Ok(Statement::If {
                        expression,
                        if_body,
                        else_body,
                    });
                }
                Token::For => todo!(),
                _ => return Err(Error::Syntax(next)),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::view::template::tokenizer::{Tokenize, Value};

    #[test]
    fn test_statements_basic() -> Result<(), Error> {
        let t1 = r#"<% if variable == 5 %>right<% else %>wrong<% end %>"#.tokenize()?;

        let ast = Statement::parse(&mut t1.into_iter().peekable())?;
        let mut context = Context::default();
        context.set("variable", &Value::Integer(5));

        let value = ast.evaluate(&context)?;
        assert!(value == "right");

        Ok(())
    }
}

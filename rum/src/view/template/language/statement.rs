use super::{
    super::{Context, Error, Token, TokenWithContext, Tokenize, Value},
    Expression, Term,
};
use std::iter::{Iterator, Peekable};

macro_rules! expect {
    ($got:expr, $expected:expr) => {
        if $got.token() != $expected {
            // println!("{}:{}", file!(), line!());
            return Err(Error::WrongToken($got, $expected));
        }
    };
}

macro_rules! block_end {
    ($iter:expr) => {
        while let Some(token) = $iter.next() {
            expect!(token, Token::BlockEnd);
            break;
        }
    };
}

#[derive(Debug, Clone)]
pub enum Statement {
    // e.g. `<%= variable %>`
    Print(Expression),
    // e.g. `<html><body></body></html>`
    PrintText(String),
    // e.g. `<% if variable == 5 %>right<% else %>wrong<% end %>`
    If {
        expression: Expression,
        if_body: Vec<Statement>,
        else_body: Vec<Statement>,
        else_if: bool,
    },

    // `<% else %>`
    Else,
    // `<% end %>
    End,

    // `<% for var in [1, 2, 3] %> <%= var %> <% end %>`
    For {
        variable: Term,
        list: Expression,
        body: Vec<Statement>,
    },
}

impl Statement {
    pub fn from_str(string: &str) -> Result<Self, Error> {
        let tokens = string.tokenize()?;
        Statement::parse(&mut tokens.into_iter().peekable())
    }

    pub fn evaluate(&self, context: &Context) -> Result<String, Error> {
        match self {
            Statement::PrintText(text) => Ok(text.clone()),
            Statement::If {
                expression,
                if_body,
                else_body,
                ..
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
            Statement::Print(expression) => Ok(expression.evaluate(context)?.to_string()),
            Statement::For {
                variable,
                list,
                body,
            } => {
                let mut result = String::new();
                let list = list.evaluate(context)?;
                let mut for_context = context.clone();
                match list {
                    Value::List(values) => {
                        for value in values {
                            match variable {
                                // Convert the variable to a value from the list.
                                Term::Variable(name) => {
                                    for_context.set(&name, value)?;
                                }
                                Term::Constant(_) => (), // Looks like just a loop with no variables
                                _ => todo!(),            // Function call is interesting
                            };

                            for statement in body {
                                result.push_str(&statement.evaluate(&for_context)?);
                            }
                        }
                    }

                    _ => return Err(Error::Syntax(TokenWithContext::new(Token::End, 0, 0))),
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
                Token::BlockStart | Token::BlockEnd => (),
                Token::BlockStartPrint => {
                    let expression = Expression::parse(iter)?;
                    block_end!(iter);
                    return Ok(Statement::Print(expression));
                }
                Token::Else => {
                    block_end!(iter);
                    return Ok(Statement::Else);
                }
                Token::If | Token::ElseIf => {
                    let else_if = next.token() == Token::ElseIf;
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
                                    else_if,
                                })
                            }

                            Statement::If { else_if: true, .. } => {
                                else_body.push(statement);
                                break;
                                // if
                                // elsif
                                // elsif
                                // else
                                // end
                                //
                                // translates into this:
                                //
                                // if
                                // else
                                //   if
                                //   else
                                //     if
                                //     else
                                //     end
                                //   end
                                // end
                            }

                            Statement::Else => loop {
                                let statement = Statement::parse(iter)?;

                                match statement {
                                    Statement::End => {
                                        return Ok(Statement::If {
                                            expression,
                                            if_body,
                                            else_body,
                                            else_if,
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
                        else_if,
                    });
                }

                Token::For => {
                    let variable = Expression::parse(iter)?;
                    let term = match variable {
                        Expression::Term { term } => term,
                        _ => return Err(Error::Syntax(next)),
                    };

                    let in_ = iter.next().ok_or(Error::Eof)?;
                    expect!(in_, Token::In);

                    let list = Expression::parse(iter)?;
                    block_end!(iter);

                    let mut body = vec![];

                    loop {
                        let statement = Statement::parse(iter)?;

                        match statement {
                            Statement::End => {
                                break;
                            }
                            statement => body.push(statement),
                        }
                    }

                    return Ok(Statement::For {
                        variable: term,
                        list,
                        body,
                    });
                }
                _ => return Err(Error::Syntax(next)),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::view::template::lexer::{Tokenize, Value};

    #[test]
    fn test_statements_basic() -> Result<(), Error> {
        let t1 = r#"<% if variable == 5 %>right<% else %>wrong<% end %>"#.tokenize()?;

        let ast = Statement::parse(&mut t1.into_iter().peekable())?;
        let mut context = Context::default();
        context.set("variable", Value::Integer(5))?;

        let value = ast.evaluate(&context)?;
        assert!(value == "right");

        Ok(())
    }

    #[test]
    fn test_statements_if_else() -> Result<(), Error> {
        let t1 = "<% if variable == 5 %>
                right
            <% elsif variable == 6 %>
                wrong
            <% else %>
                neither
            <% end %>"
            .tokenize()?;
        let ast = Statement::parse(&mut t1.into_iter().peekable()).unwrap();
        let mut context = Context::default();
        context.set("variable", Value::Integer(7))?;
        let result = ast.evaluate(&context)?;
        assert_eq!(result.trim(), "neither");

        Ok(())
    }

    #[test]
    fn test_print_expression() -> Result<(), Error> {
        let t1 = "<%= variable %>";
        let mut context = Context::default();
        context.set("variable", Value::Integer(7))?;

        let ast = Statement::parse(&mut t1.tokenize()?.into_iter().peekable())?;
        let result = ast.evaluate(&context)?;
        assert_eq!(result, "7");

        Ok(())
    }

    #[test]
    fn test_for_loop() -> Result<(), Error> {
        let t1 = r#"<% for a in [1, "hello", 3.45, variable] %><li><%= a %></li><% end %>"#
            .tokenize()?;
        let mut context = Context::default();
        context.set("variable", Value::String("variable value".into()))?;
        let ast = Statement::parse(&mut t1.into_iter().peekable())?;
        let result = ast.evaluate(&context)?;

        assert_eq!(
            result,
            "<li>1</li><li>hello</li><li>3.45</li><li>variable value</li>"
        );

        let result = Statement::from_str(
            "
<% for v in [1, 2, 3].enumerate %>
<p><%= v.0 + 1 %>. <%= v.1 %></p>
<% end %>
        "
            .trim(),
        )?
        .evaluate(&Context::default())?;

        assert_eq!(result, "<p>1. 1</p><p>2. 2</p><p>3. 3</p>");

        Ok(())
    }
}

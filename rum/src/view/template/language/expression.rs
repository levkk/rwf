use super::{
    super::lexer::{Token, TokenWithContext, Value},
    super::Context,
    super::Error,
    Op, Term,
};

use std::iter::{Iterator, Peekable};

/// An expression, like `5 == 6` or `logged_in == false`,
/// which when evaluated produces a single value, e.g. `true`.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Standard `5 + 6`-style expression.
    // It's recusive, so you can have something like `(5 + 6) / (1 - 5)`.
    Binary {
        left: Box<Expression>,
        op: Op,
        right: Box<Expression>,
    },

    Unary {
        op: Op,
        operand: Box<Expression>,
    },

    // Base case for recursive expression parsing, which evaluates to the value
    // of the term, e.g. `5` evalutes to `5` or `variable_name` evalutes to whatever
    // the variable is set to in the context.
    Term {
        term: Term,
    },

    // A list of expressions, e.g.
    // `[1, 2, variable, "hello world"]`
    //
    // The list is dynamically evaluated at runtime, so it can contain variables
    // and constants, as long as the variable is in scope.
    List {
        terms: Vec<Expression>,
    },
}

impl Expression {
    /// Create new constant expression (term).
    pub fn constant(value: Value) -> Self {
        Self::Term {
            term: Term::constant(value),
        }
    }

    /// Create new variable expression (term).
    pub fn variable(variable: String) -> Self {
        Self::Term {
            term: Term::variable(variable),
        }
    }

    /// Evaluate the expression to a value given the context.
    pub fn evaluate(&self, context: &Context) -> Result<Value, Error> {
        match self {
            Expression::Term { term } => term.evaluate(context),
            Expression::Binary { left, op, right } => {
                let left = left.evaluate(context)?;
                let right = right.evaluate(context)?;
                op.evaluate_binary(&left, &right)
            }
            Expression::List { terms } => {
                let mut list = vec![];
                for term in terms {
                    list.push(term.evaluate(context)?);
                }
                Ok(Value::List(list))
            }
            _ => todo!(),
        }
    }

    /// Recusively parse the expression.
    ///
    /// Consumes language tokens automatically.
    ///
    /// TODO: handle paranthesis and multi-term expressions, e.g. `5 + 6 && 7 || true`.
    pub fn parse(
        iter: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    ) -> Result<Self, Error> {
        loop {
            let next = iter.next().ok_or(Error::Eof)?;

            match next.token() {
                // Helps with testing, but these tokens shouldn't be passed
                // to the expression parser.
                Token::BlockStart | Token::BlockEnd => (),
                Token::Variable(name) => {
                    let left = Self::variable(name);
                    let next = iter.peek().ok_or(Error::Eof)?;

                    match Op::from_token(next.token()) {
                        Some(op) => {
                            let _ = iter.next().ok_or(Error::Eof)?;
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
                    let next = iter.peek().ok_or(Error::Eof)?;
                    match Op::from_token(next.token()) {
                        Some(op) => {
                            let _ = iter.next().ok_or(Error::Eof)?;
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

                Token::SquareBracketStart => {
                    let mut terms = vec![];

                    loop {
                        let next = iter.next().ok_or(Error::Eof)?;
                        match next.token() {
                            Token::SquareBracketEnd => break,
                            Token::Comma => continue,
                            Token::Value(value) => terms.push(Expression::constant(value)),
                            Token::Variable(variable) => terms.push(Expression::variable(variable)),
                            _ => return Err(Error::Syntax(next)),
                        }
                    }

                    return Ok(Expression::List { terms });
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

        let t2 = "<% 1 && 1 %>".tokenize()?;
        let expr = Expression::parse(&mut t2.into_iter().peekable())?;
        let value = expr.evaluate(&Context::default())?;
        assert_eq!(value, Value::Boolean(true));

        Ok(())
    }

    #[test]
    fn test_list() -> Result<(), Error> {
        let t1 = r#"<% [1, 2, "hello", 3.13, variable] %>"#.tokenize()?;
        let ast = Expression::parse(&mut t1.into_iter().peekable())?;
        println!("{:?}", ast);

        Ok(())
    }
}

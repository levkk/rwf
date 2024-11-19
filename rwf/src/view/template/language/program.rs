//! Executable template.
//!
//! A program is a list of statements.
use super::super::{Context, Error, TokenWithContext, Tokenize};
use super::Statement;

/// Executable program.
#[derive(Debug, Clone)]
pub struct Program {
    statements: Vec<Statement>,
}

impl Program {
    /// Evaluate the program given the context. The context contains variable definitions.
    pub fn evaluate(&self, context: &Context) -> Result<String, Error> {
        let mut result = String::new();
        for statement in &self.statements {
            result.push_str(&statement.evaluate(context)?);
        }

        Ok(result)
    }

    /// Parse the program from a list of tokens.
    pub fn parse(tokens: Vec<TokenWithContext>) -> Result<Self, Error> {
        let mut iter = tokens.into_iter().peekable();
        let mut statements = vec![];

        while iter.peek().is_some() {
            let statement = Statement::parse(&mut iter)?;
            statements.push(statement);
        }

        Ok(Program { statements })
    }

    /// Compile the program from source.
    pub fn from_str(source: &str) -> Result<Self, Error> {
        let tokens = source.tokenize()?;
        Program::parse(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::view::template::{Tokenize, Value};
    use std::collections::HashMap;

    #[test]
    fn test_basic_program() -> Result<(), Error> {
        let program =
            "<html><body><% if 1 == 4 %>world is great<% else %>not so much<% end %></body></html>"
                .tokenize()?;
        let program = Program::parse(program)?;
        let output = program.evaluate(&Context::default())?;
        assert_eq!("<html><body>not so much</body></html>", output);
        Ok(())
    }

    #[test]
    fn test_program_print() -> Result<(), Error> {
        let program = r#"
            <html>
                <head>
                    <title><%= 5 %></title>
                </head>
            </html>
        "#
        .tokenize()?;
        let ast = Program::parse(program)?;
        println!("{:?}", ast);

        Ok(())
    }

    #[test]
    fn test_secure_links() -> Result<(), Error> {
        let program = r#"
            <a href="/api/users/<%= encrypt_number(user.id) %>"><%= user.email %></a>"#
            .tokenize()?;
        let user = HashMap::from([
            (String::from("id"), Value::Integer(25)),
            (String::from("email"), Value::String("test@test.com".into())),
        ]);

        let mut context = Context::new();
        context.set("user", Value::Hash(user))?;

        let ast = Program::parse(program)?;
        let result = ast.evaluate(&context)?;

        // Make sure the "uuid" is there.
        assert_eq!(result.chars().filter(|c| *c == '-').count(), 3);

        Ok(())
    }
}

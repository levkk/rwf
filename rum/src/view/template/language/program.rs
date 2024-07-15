use super::super::{Context, Error, TokenWithLine};
use super::Statement;

#[derive(Debug)]
pub struct Program {
    statements: Vec<Statement>,
}

impl Program {
    pub fn evaluate(&self, context: &Context) -> Result<String, Error> {
        let mut result = String::new();
        for statement in &self.statements {
            result.push_str(&statement.evaluate(context)?);
        }

        Ok(result)
    }

    pub fn parse(tokens: Vec<TokenWithLine>) -> Result<Self, Error> {
        let mut iter = tokens.into_iter().peekable();
        let mut statements = vec![];

        while iter.peek().is_some() {
            let statement = Statement::parse(&mut iter)?;
            statements.push(statement);
        }

        Ok(Program { statements })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::view::template::{Tokenize, Tokenizer};

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
}

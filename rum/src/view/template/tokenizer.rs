use crate::model::Error;

pub static START: &str = "<%";
pub static END: &str = "%>";
pub static CONTROL: &[char] = &['<', '%'];

#[derive(Debug)]
pub enum Token {
    String(String),
    If,
    ElseIf,
    Else,
    BlockStart,
    BlockEnd,
    BlockStartPrint,
}

pub struct Tokenizer<'a> {
    source: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn tokens(&mut self) -> Result<Vec<Token>, Error> {
        let mut buffer = String::new();
        let mut control = String::new();
        let mut tokens = vec![];
        let mut control_started = false;

        let save_buffer = |buffer: &mut String, tokens: &mut Vec<Token>| {
            if !buffer.is_empty() {
                tokens.push(Token::String(std::mem::take(buffer)));
            }
        };

        for c in self.source.chars() {
            match c {
                '<' | '%' | '=' => {
                    control.push(c);
                }

                c => match control.len() {
                    1 => (),
                    2 => match control.as_str() {
                        "%>" => {
                            tokens.push(Token::BlockEnd);
                            control.clear();
                            control_started = false;
                        }
                        token => {
                            if control_started {
                                return Err(Error::UnknownToken(token.to_string()));
                            } else {
                                buffer.extend(token.chars());
                                control.clear();
                            }
                        }
                    },
                    3 => match control.as_str() {
                        "<%=" => {
                            control_started = true;
                            tokens.push(Token::BlockStartPrint);
                            control.clear();
                        }

                        "<% " => {
                            control_started = true;
                            tokens.push(Token::BlockStart);
                            control.clear();
                        }

                        token => {
                            buffer.extend(token.chars());
                            control.clear();
                        }
                    },

                    _ => {
                        buffer.push(c);
                    }
                },
            };
        }

        // save_buffer(&mut buffer, &mut tokens);

        //     if buffer == START {
        //         if !buffer.is_empty() {
        //             tokens.push(Token::String(std::mem::take(&mut buffer)));
        //         }
        //         tokens.push(Token::BlockStart);
        //         buffer.clear();
        //     } else if buffer == END {
        //         tokens.push(Token::BlockEnd);
        //         buffer.clear()
        //     }
        // }

        // if !buffer.is_empty() {
        //     tokens.push(Token::String(buffer));
        // }

        Ok(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize_basic() -> Result<(), Error> {
        let template = r#"
            <html>
                <head>
                    <title><%= title %></title>
                </head>
                <body>
                    <h1>Hello world</h1>
                </body>
            </html>
        "#;

        let tokens = Tokenizer::new(&template).tokens()?;
        println!("{:?}", tokens);

        Ok(())
    }
}

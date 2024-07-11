use crate::model::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Text(String),
    Variable(String),
    String(String),
    StringStart,
    StringEnd,
    If,
    ElseIf,
    Else,
    EndIf,
    Integer(i64),
    Float(f64),
    BlockStart,
    BlockStartPrint,
    BlockEnd,
    BlockStartBracket,
    BlockEndBracket,
    BlockSign,
    Print,
    Space,
    Character(char),
    Comparison(Comparison),
    Dot,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Comparison {
    Equals,
    NotEquals,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
}

pub struct Tokenizer<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    buffer: String,
    code_block: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: vec![],
            buffer: String::new(),
            code_block: false,
        }
    }

    pub fn tokens(mut self) -> Result<Vec<Token>, Error> {
        let mut iter = self.source.chars();

        while let Some(c) = iter.next() {
            match c {
                '<' => {
                    let n = iter.next();

                    match n {
                        Some('%') => {
                            let m = iter.next();

                            match m {
                                Some('=') => {
                                    self.drain_buffer();
                                    self.tokens.push(Token::BlockStartPrint);
                                    self.code_block = true;
                                }

                                Some(c) => {
                                    self.drain_buffer();
                                    self.tokens.push(Token::BlockStart);

                                    match c {
                                        ' ' => self.tokens.push(Token::Space),
                                        c => self.buffer.push(c),
                                    }

                                    self.code_block = true;
                                }

                                None => {
                                    self.drain_buffer();
                                    self.tokens.push(Token::BlockStart);
                                }
                            }
                        }

                        Some(c) => {
                            self.buffer.push('<');
                            self.buffer.push(c);
                        }

                        None => (),
                    }
                }

                '.' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(Token::Dot);
                    } else {
                        self.buffer.push('.');
                    }
                }

                '%' => {
                    let n = iter.next();

                    match n {
                        Some('>') => {
                            self.drain_buffer();
                            self.tokens.push(Token::BlockEnd);
                            self.code_block = false;
                        }

                        Some(c) => {
                            self.buffer.push('%');
                            self.buffer.push(c);
                        }

                        None => (),
                    }
                }

                '"' => {
                    if self.code_block {
                        self.drain_buffer();
                        let mut string = String::new();

                        while let Some(c) = iter.next() {
                            match c {
                                '"' => {
                                    self.tokens.push(Token::String(string));
                                    break;
                                }

                                _ => string.push(c),
                            }
                        }
                    } else {
                        self.buffer.push('"');
                    }
                }

                ' ' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(Token::Space);
                    } else {
                        self.buffer.push(' ');
                    }
                }

                c => self.buffer.push(c),
            }
        }

        self.drain_buffer();

        Ok(self.tokens)
    }

    fn drain_buffer(&mut self) {
        if !self.buffer.is_empty() {
            let s = std::mem::take(&mut self.buffer);
            if self.code_block {
                match s.as_str() {
                    "if" => self.tokens.push(Token::If),
                    "else" => self.tokens.push(Token::Else),
                    "else if" => self.tokens.push(Token::ElseIf),
                    "end" => self.tokens.push(Token::EndIf),
                    "==" => self.tokens.push(Token::Comparison(Comparison::Equals)),
                    st => {
                        if let Ok(integer) = st.parse::<i64>() {
                            self.tokens.push(Token::Integer(integer));
                        } else if let Ok(float) = st.parse::<f64>() {
                            self.tokens.push(Token::Float(float));
                        } else {
                            self.tokens.push(Token::Variable(s));
                        }
                    }
                }
            } else {
                self.tokens.push(Token::Text(s));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize_basic() -> Result<(), Error> {
        let template = r#"<title><%= title %></title><body><% if variable == 1 %></body><% if variable == "string" %>"#;
        let tokens = Tokenizer::new(&template).tokens()?;
        assert_eq!(
            tokens,
            vec![
                Token::Text("<title>".into()),
                Token::BlockStartPrint,
                Token::Space,
                Token::Variable("title".into()),
                Token::Space,
                Token::BlockEnd,
                Token::Text("</title><body>".into()),
                Token::BlockStart,
                Token::Space,
                Token::If,
                Token::Space,
                Token::Variable("variable".into()),
                Token::Space,
                Token::Comparison(Comparison::Equals),
                Token::Space,
                Token::Integer(1),
                Token::Space,
                Token::BlockEnd,
                Token::Text("</body>".into()),
                Token::BlockStart,
                Token::If,
                Token::Variable("variable".into()),
                Token::Space,
                Token::Comparison(Comparison::Equals),
                Token::Space,
                Token::String("string".into()),
            ]
        );

        Ok(())
    }
}

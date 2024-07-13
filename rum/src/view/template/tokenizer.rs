use crate::model::Error;

#[derive(Debug, Clone)]
pub struct TokenWithLine {
    pub token: Token,
    pub line: usize,
}

impl std::ops::Deref for TokenWithLine {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

impl TokenWithLine {
    pub fn new(token: Token, line: usize) -> Self {
        Self { token, line }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn token(self) -> Token {
        self.token
    }
}

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
    And,
    Or,
    Not,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Comparison {
    Equals,
    NotEquals,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
    Not,
    True,
    False,
}

pub struct Tokenizer<'a> {
    source: &'a str,
    tokens: Vec<TokenWithLine>,
    buffer: String,
    code_block: bool,
    line: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: vec![],
            buffer: String::new(),
            code_block: false,
            line: 1,
        }
    }

    pub fn tokens(mut self) -> Result<Vec<TokenWithLine>, Error> {
        let mut iter = self.source.chars();

        while let Some(c) = iter.next() {
            match c {
                '\n' => self.line += 1,
                '<' => {
                    let n = iter.next();

                    match n {
                        Some('%') => {
                            let m = iter.next();

                            match m {
                                Some('=') => {
                                    self.drain_buffer();
                                    self.tokens.push(self.add_token(Token::BlockStartPrint));
                                    self.code_block = true;
                                }

                                Some(c) => {
                                    self.drain_buffer();
                                    self.tokens.push(self.add_token(Token::BlockStart));

                                    match c {
                                        ' ' => self.tokens.push(self.add_token(Token::Space)),
                                        c => self.buffer.push(c),
                                    }

                                    self.code_block = true;
                                }

                                None => {
                                    self.drain_buffer();
                                    self.tokens.push(self.add_token(Token::BlockStart));
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
                        self.tokens.push(self.add_token(Token::Dot));
                    } else {
                        self.buffer.push('.');
                    }
                }

                '!' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::Not));
                    } else {
                        self.buffer.push('!');
                    }
                }

                '%' => {
                    let n = iter.next();

                    match n {
                        Some('>') => {
                            self.drain_buffer();
                            self.tokens.push(self.add_token(Token::BlockEnd));
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
                                    self.tokens.push(self.add_token(Token::String(string)));
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
                        self.tokens.push(self.add_token(Token::Space));
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
                    "if" => self.tokens.push(self.add_token(Token::If)),
                    "else" => self.tokens.push(self.add_token(Token::Else)),
                    "elsif" => self.tokens.push(self.add_token(Token::ElseIf)),
                    "end" => self.tokens.push(self.add_token(Token::EndIf)),
                    "&&" => self.tokens.push(self.add_token(Token::And)),
                    "||" => self.tokens.push(self.add_token(Token::Or)),
                    "==" => self
                        .tokens
                        .push(self.add_token(Token::Comparison(Comparison::Equals))),
                    st => {
                        if let Ok(integer) = st.parse::<i64>() {
                            self.tokens.push(self.add_token(Token::Integer(integer)));
                        } else if let Ok(float) = st.parse::<f64>() {
                            self.tokens.push(self.add_token(Token::Float(float)));
                        } else {
                            self.tokens.push(self.add_token(Token::Variable(s)));
                        }
                    }
                }
            } else {
                self.tokens.push(self.add_token(Token::Text(s)));
            }
        }
    }

    fn add_token(&self, token: Token) -> TokenWithLine {
        TokenWithLine::new(token, self.line)
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
            tokens
                .into_iter()
                .map(|token| token.token().clone())
                .collect::<Vec<_>>(),
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

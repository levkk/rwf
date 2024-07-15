use crate::view::template::Error;

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

    pub fn token(&self) -> Token {
        self.token.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Text(String),
    Variable(String),
    String(String),
    Value(Value),
    StringStart,
    StringEnd,
    If,
    ElseIf,
    Else,
    End,
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
    For,
    In,
    Do,
    Plus,
    Minus,
    Mod,
    Div,
    Mult,
    Equals,
    NotEquals,
    GreaterThan,
    GreaterEqualThan,
    LessThan,
    LessEqualThan,
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
    number: bool,
    line: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: vec![],
            buffer: String::new(),
            code_block: false,
            number: false,
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
                    if self.code_block && !self.number {
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
                            if self.code_block {
                                self.drain_buffer();
                                self.tokens.push(self.add_token(Token::BlockEnd));
                                self.code_block = false;
                            } else {
                                self.buffer.push('%');
                                self.buffer.push('>');
                            }
                        }

                        Some(c) => {
                            if self.code_block {
                                self.drain_buffer();
                                self.tokens.push(self.add_token(Token::Mod));
                            } else {
                                self.buffer.push('%');
                                self.buffer.push(c);
                            }
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
                                    self.tokens
                                        .push(self.add_token(Token::Value(Value::String(string))));
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

                '+' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::Plus));
                    } else {
                        self.buffer.push('+');
                    }
                }

                '-' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::Minus));
                    } else {
                        self.buffer.push('-');
                    }
                }

                '*' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::Mult));
                    } else {
                        self.buffer.push('*');
                    }
                }

                '/' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::Div));
                    } else {
                        self.buffer.push('/');
                    }
                }

                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    if self.code_block {
                        self.number = true;
                        self.buffer.push(c);
                    } else {
                        self.buffer.push(c);
                    }
                }

                c => self.buffer.push(c),
            }
        }

        self.drain_buffer();

        Ok(self
            .tokens
            .into_iter()
            .filter(|token| &token.token != &Token::Space)
            .collect())
    }

    fn drain_buffer(&mut self) {
        if !self.buffer.is_empty() {
            let s = std::mem::take(&mut self.buffer);
            if self.code_block {
                match s.as_str() {
                    "if" => self.tokens.push(self.add_token(Token::If)),
                    "else" => self.tokens.push(self.add_token(Token::Else)),
                    "elsif" => self.tokens.push(self.add_token(Token::ElseIf)),
                    "end" => self.tokens.push(self.add_token(Token::End)),
                    "for" => self.tokens.push(self.add_token(Token::For)),
                    "in" => self.tokens.push(self.add_token(Token::In)),
                    "do" => self.tokens.push(self.add_token(Token::Do)),
                    "&&" => self.tokens.push(self.add_token(Token::And)),
                    "||" => self.tokens.push(self.add_token(Token::Or)),
                    "==" => self.tokens.push(self.add_token(Token::Equals)),
                    "!=" => self.tokens.push(self.add_token(Token::NotEquals)),
                    ">" => self.tokens.push(self.add_token(Token::GreaterThan)),
                    ">=" => self.tokens.push(self.add_token(Token::GreaterEqualThan)),
                    "<" => self.tokens.push(self.add_token(Token::LessThan)),
                    "<=" => self.tokens.push(self.add_token(Token::LessEqualThan)),
                    st => {
                        if let Ok(integer) = st.parse::<i64>() {
                            self.tokens
                                .push(self.add_token(Token::Value(Value::Integer(integer))));
                        } else if let Ok(float) = st.parse::<f64>() {
                            self.tokens
                                .push(self.add_token(Token::Value(Value::Float(float))));
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

pub trait Tokenize {
    fn tokenize(&self) -> Result<Vec<TokenWithLine>, Error>;
}

impl Tokenize for &str {
    fn tokenize(&self) -> Result<Vec<TokenWithLine>, Error> {
        Tokenizer::new(self).tokens()
    }
}

impl Tokenize for String {
    fn tokenize(&self) -> Result<Vec<TokenWithLine>, Error> {
        Tokenizer::new(self).tokens()
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
                Token::Equals,
                Token::Space,
                Token::Integer(1),
                Token::Space,
                Token::BlockEnd,
                Token::Text("</body>".into()),
                Token::BlockStart,
                Token::If,
                Token::Variable("variable".into()),
                Token::Space,
                Token::Equals,
                Token::Space,
                Token::String("string".into()),
            ]
        );

        Ok(())
    }
}

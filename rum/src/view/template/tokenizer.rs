use super::Error;

#[derive(Debug, Clone)]
pub struct TokenWithContext {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

impl std::ops::Deref for TokenWithContext {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

impl TokenWithContext {
    pub fn new(token: Token, line: usize, column: usize) -> Self {
        Self {
            token,
            line,
            column,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn token(&self) -> Token {
        self.token.clone()
    }
}

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Null,
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            _ => true,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Text(String),
    Variable(String),
    String(String),
    Value(Value),
    If,
    ElseIf,
    Else,
    End,
    BlockStart,
    BlockStartPrint,
    BlockEnd,
    BlockSign,
    Print,
    Space,
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
    tokens: Vec<TokenWithContext>,
    buffer: String,
    code_block: bool,
    number: bool,
    line: usize,
    column: usize,
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
            column: 1,
        }
    }

    pub fn tokens(mut self) -> Result<Vec<TokenWithContext>, Error> {
        let mut iter = self.source.chars();

        while let Some(c) = iter.next() {
            self.column += 1;
            match c {
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                }
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
                        let next = iter.next();
                        match next {
                            Some('=') => {
                                self.drain_buffer();
                                self.tokens.push(self.add_token(Token::NotEquals));
                            }

                            Some(' ') => {
                                self.tokens.push(self.add_token(Token::Space));
                            }

                            Some(c) => {
                                self.tokens.push(self.add_token(Token::Not));
                                self.buffer.push(c);
                            }

                            None => return Err(Error::Eof),
                        }
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

    fn add_token(&self, token: Token) -> TokenWithContext {
        TokenWithContext::new(token, self.line, self.column)
    }
}

pub trait Tokenize {
    fn tokenize(&self) -> Result<Vec<TokenWithContext>, Error>;
}

impl Tokenize for &str {
    fn tokenize(&self) -> Result<Vec<TokenWithContext>, Error> {
        Tokenizer::new(self).tokens()
    }
}

impl Tokenize for String {
    fn tokenize(&self) -> Result<Vec<TokenWithContext>, Error> {
        Tokenizer::new(self).tokens()
    }
}

#[cfg(test)]
mod test {}

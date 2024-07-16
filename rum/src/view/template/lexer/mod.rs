pub mod token;
pub mod value;

pub use token::Token;
pub use value::{ToValue, Value};

use super::Error;

#[derive(Debug, Clone)]
pub struct TokenWithContext {
    token: Token,
    line: usize,
    column: usize,
}

impl std::fmt::Display for TokenWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:?} (line: {}, column: {})",
            self.token, self.line, self.column
        )
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

/// The lexer converts a template text
/// into a list of tokens that may mean something
/// in our template language.
///
/// Since we're parsing templates, anything that's
/// not inside a code block, e.g. `<% %>` is considered
/// to be just text that needs to be printed as-is.
///
/// This text is represented by the special `Token::Text`.
pub struct Lexer<'a> {
    // Template source.
    source: &'a str,
    // Resulting tokens.
    tokens: Vec<TokenWithContext>,
    // Buffer for multi-character tokens.
    buffer: String,
    // Indicates if we're inside code block where
    // some characters have special meaning, e.g. `<% 5 / 3 %>`
    code_block: bool,
    // Indicates we're currently parsing a number, so the `.` character
    // has special meaning.
    number: bool,
    // Which line we're on.
    line: usize,
    // Which column we're on. The parser processes input one character at a time.
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Create new lexer from text input.
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

    /// Parse an input string into tokens supported by our template language.
    ///
    /// Tokens are processed one character at a time. Multi-character tokens like `if`
    /// or `for` are buffered and parsed as a string.
    pub fn tokens(mut self) -> Result<Vec<TokenWithContext>, Error> {
        let mut iter = self.source.chars();

        while let Some(c) = iter.next() {
            self.column += 1;
            match c {
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                }
                '\r' => {
                    self.column -= 1;
                } // Handle column count on Windows.

                // Possibly a code block start tag.
                '<' => {
                    let n = iter.next();

                    match n {
                        Some('%') => {
                            let m = iter.next();

                            match m {
                                // `<%=` (print expression)
                                Some('=') => {
                                    self.drain_buffer();
                                    self.tokens.push(self.add_token(Token::BlockStartPrint));
                                    self.code_block = true;
                                }

                                // `<%` (code block start)
                                Some(c) => {
                                    self.drain_buffer();
                                    self.tokens.push(self.add_token(Token::BlockStart));

                                    match c {
                                        ' ' => self.tokens.push(self.add_token(Token::Space)),
                                        c => self.buffer.push(c),
                                    }

                                    self.code_block = true;
                                }

                                // Dangling code block start bracket. Syntax error,
                                // but we don't need to handle this here.
                                None => {
                                    self.drain_buffer();
                                    self.tokens.push(self.add_token(Token::BlockStart));
                                }
                            }
                        }

                        // Nothing, just a "less than" sign, e.g. opening bracket for an HTML tag.
                        Some(c) => {
                            self.buffer.push('<');
                            self.buffer.push(c);
                        }

                        None => (),
                    }
                }

                '.' => {
                    // If we're parsing a number, keep the dot for the floating point
                    // notation. Otherwise, it's an accessor for a method call or object attribute.
                    if self.code_block {
                        if self.number {
                            let next = iter.next();
                            match next {
                                Some(c) => {
                                    if c.is_numeric() {
                                        self.buffer.push('.');
                                        self.buffer.push(c);
                                    } else {
                                        self.drain_buffer();
                                        self.tokens.push(self.add_token(Token::Dot));
                                        self.buffer.push(c);
                                    }
                                }

                                None => {
                                    self.drain_buffer();
                                    self.tokens.push(self.add_token(Token::Dot));
                                }
                            }
                        } else {
                            self.drain_buffer();
                            self.tokens.push(self.add_token(Token::Dot));
                        }
                    } else {
                        // Or it's just a dot part of the template.
                        self.buffer.push('.');
                    }
                }

                '!' => {
                    if self.code_block {
                        let next = iter.next();
                        match next {
                            // `<% != %>`
                            Some('=') => {
                                self.drain_buffer();
                                self.tokens.push(self.add_token(Token::NotEquals));
                            }

                            Some(c) => {
                                if c == ' ' {
                                    self.tokens.push(self.add_token(Token::Space));
                                } else {
                                    self.buffer.push(c);
                                }

                                self.tokens.push(self.add_token(Token::Not));
                            }

                            None => return Err(Error::Eof),
                        }
                    } else {
                        // Just a !, e.g `<h1>oh hello there!</h1>`
                        self.buffer.push('!');
                    }
                }

                // Potentially a code block end tag.
                '%' => {
                    let n = iter.next();

                    match n {
                        Some('>') => {
                            // We are parsing a code block, so this tells us the code is over.
                            if self.code_block {
                                self.drain_buffer();
                                self.tokens.push(self.add_token(Token::BlockEnd));
                                self.code_block = false;
                            } else {
                                // Just a random `%>` tag in the template, means nothing
                                // without a starting tag.
                                self.buffer.push('%');
                                self.buffer.push('>');
                            }
                        }

                        Some(c) => {
                            // If we're parsing code, then this is a modulus operator, e.g. `5 % 3 == 2`
                            if self.code_block {
                                self.drain_buffer();
                                self.tokens.push(self.add_token(Token::Mod));
                            } else {
                                self.buffer.push('%');
                                self.buffer.push(c);
                            }
                        }

                        None => {
                            // A mod operator with nothing after it. Syntax error,
                            // but we don't need to handle it here.
                            if self.code_block {
                                self.drain_buffer();
                                self.tokens.push(self.add_token(Token::Mod));
                            } else {
                                // A template ending with a "%" for some reason.
                                self.buffer.push('%');
                            }
                        }
                    }
                }

                // Maybe a string.
                '"' => {
                    // We're parsing a string, e.g. `<% "hello world" %>`.
                    // TODO: handle escape characters.
                    if self.code_block {
                        self.drain_buffer();
                        let mut string = String::new();

                        // Look for the closing `"`
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
                        // Just a double quote, e.g. `<html lang="en-US"%>`
                        self.buffer.push('"');
                    }
                }

                ' ' => {
                    // Spaces separate tokens.
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::Space));
                    } else {
                        // Spaces separate words.
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

                '[' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::SquareBracketStart));
                    } else {
                        self.buffer.push(c);
                    }
                }

                ',' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::Comma));
                    } else {
                        self.buffer.push(c);
                    }
                }

                ']' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::SquareBracketEnd));
                    } else {
                        self.buffer.push(c);
                    }
                }

                '(' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::RoundBracketStart));
                    } else {
                        self.buffer.push(c);
                    }
                }

                ')' => {
                    if self.code_block {
                        self.drain_buffer();
                        self.tokens.push(self.add_token(Token::RoundBracketEnd));
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
            // Remove spaces from output, the lexer handled it, the parser doesn't need to.
            .filter(|token| &token.token != &Token::Space)
            .collect())
    }

    // Handle multi-character tokens.
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
                    "true" => self
                        .tokens
                        .push(self.add_token(Token::Value(Value::Boolean(true)))),
                    "false" => self
                        .tokens
                        .push(self.add_token(Token::Value(Value::Boolean(false)))),
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

    // Add token to output with lexer context (e.g. line number).
    fn add_token(&self, token: Token) -> TokenWithContext {
        TokenWithContext::new(token, self.line, self.column)
    }
}

// Easily tokenize strings.
pub trait Tokenize {
    // Parse a string and convert it to a list of tokens.
    fn tokenize(&self) -> Result<Vec<TokenWithContext>, Error>;
}

impl Tokenize for &str {
    fn tokenize(&self) -> Result<Vec<TokenWithContext>, Error> {
        Lexer::new(self).tokens()
    }
}

impl Tokenize for String {
    fn tokenize(&self) -> Result<Vec<TokenWithContext>, Error> {
        Lexer::new(self).tokens()
    }
}

#[cfg(test)]
mod test {}

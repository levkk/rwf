use super::Value;

/// A template language token, e.g. `if` or `for`.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // e.g. `<html><body></body></html>`
    Text(String),
    // e.g. `<% logged_in %>`
    Variable(String),
    // e.g. `<% "hello world" %>`
    String(String),
    // e.g. `<% 5 %>`
    Value(Value),
    // `<% if %>`
    If,
    // `<% elsif %>`
    ElseIf,
    // `<% else %>`
    Else,
    End,
    BlockStart,
    BlockStartPrint,
    BlockStartPrintRaw,
    BlockStartRender,
    BlockEnd,
    // BlockSign,
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
    SquareBracketStart,
    SquareBracketEnd,
    Comma,
    RoundBracketStart,
    RoundBracketEnd,
}

impl Token {
    pub fn len(&self) -> usize {
        match self {
            Token::If => 2,
            Token::Else => 4,
            Token::End => 3,
            _ => 0,
        }
    }
}

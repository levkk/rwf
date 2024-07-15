use super::Value;

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

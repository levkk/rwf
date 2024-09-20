#[derive(PartialEq, Clone, Debug)]
pub enum Part {
    Slash,
    Identifier(String),
    Segment(String),
}

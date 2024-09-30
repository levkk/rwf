#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Text(String),
    Data(Vec<u8>),
}

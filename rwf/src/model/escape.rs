//! Implements string escaping to prevent SQL injection attacks.
use super::value::Value;

/// Escape a user-provided value to prevent
/// SQL injection attacks.
///
/// The implementation is dependent on the value, but the
/// most common one, a [`String`] (and it's reference cousin [`&str`]), are implemented.
///
/// # Example
///
/// ```
/// use rwf::model::Escape;
///
/// let email = "guest@test.com';DROP TABLE users;";
/// assert_eq!(email.escape(), "guest@test.com'';DROP TABLE users;");
/// ```
///
pub trait Escape {
    fn escape(&self) -> String;
}

impl Escape for Value {
    fn escape(&self) -> String {
        use Value::*;

        match self {
            String(string) => string.escape(),
            Integer(integer) => format!("'{}'", integer),
            Float(float) => format!("'{}'", float),
            List(values) => format!(
                "{{{}}}", // '{1, 2, 3}'
                values
                    .iter()
                    .map(|value| value.escape())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Placeholder(number) => format!("{}", number),
            _ => todo!(),
        }
    }
}

impl Escape for String {
    fn escape(&self) -> String {
        self.replace("\"", "\"\"").replace("'", "''")
    }
}

impl Escape for &str {
    fn escape(&self) -> String {
        self.to_string().escape()
    }
}

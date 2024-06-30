use super::value::Value;

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
                    .into_iter()
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

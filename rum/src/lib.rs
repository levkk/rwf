pub mod model;
pub use tokio_postgres;

fn snake_case(string: &str) -> String {
    let mut result = "".to_string();

    for (i, c) in string.chars().enumerate() {
        if c.is_ascii_uppercase() && i != 0 {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    result
}

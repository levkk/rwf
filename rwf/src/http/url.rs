//! URL handling helpers.
//!
//! # Example
//!
//! ```
//! use rwf::http::{urlencode, urldecode};
//!
//! let url = "?foo=bar&hello=world%20";
//!
//! let decoded = urldecode(url);
//! let encoded = urlencode(&decoded);
//!
//! assert_eq!(decoded, "?foo=bar&hello=world ");
//! assert_eq!(encoded, "%3Ffoo%3Dbar%26hello%3Dworld%20");
//! ```

/// Decode a string encoded with percent-encoding, also known as URL encoding.
pub fn urldecode(s: &str) -> String {
    let mut result = String::new();
    let mut iter = s.chars().peekable();

    while let Some(c) = iter.next() {
        match c {
            '%' => {
                let mut num = String::new();

                loop {
                    match iter.peek() {
                        Some(&c)
                            if ((c.is_numeric()
                                || ['A', 'B', 'C', 'D', 'E', 'F']
                                    .contains(&c.to_ascii_uppercase()))
                                && num.len() < 2) =>
                        {
                            let _ = iter.next().unwrap();
                            num.push(c);
                        }

                        _ => {
                            let replacement = match num.to_ascii_uppercase().as_str() {
                                "3A" => ":",
                                "2F" => "/",
                                "3F" => "?",
                                "23" => "#",
                                "5B" => "[",
                                "5D" => "]",
                                "40" => "@",
                                "21" => "!",
                                "24" => "$",
                                "26" => "&",
                                "27" => "\'",
                                "28" => "(",
                                "29" => ")",
                                "2A" => "*",
                                "2B" => "+",
                                "2C" => ",",
                                "3B" => ";",
                                "3D" => "=",
                                "25" => "%",
                                "20" => " ",
                                "7B" => "{",
                                "7D" => "}",
                                "0A" => "\n",
                                _ => &num,
                            };

                            result.push_str(replacement);
                            break;
                        }
                    }
                }
            }

            '+' => result.push(' '),

            c => result.push(c),
        }
    }

    result
}

/// Encode a string using percent-encoding, also known as URL encoding.
pub fn urlencode(s: &str) -> String {
    let mut result = String::new();

    for c in s.chars() {
        let replacement = match c {
            ':' => "%3A",
            '/' => "%2F",
            '?' => "%3F",
            '#' => "%23",
            '[' => "%5B",
            ']' => "%5D",
            '@' => "%40",
            '!' => "%21",
            '$' => "%24",
            '&' => "%26",
            '\'' => "%27",
            '(' => "%28",
            ')' => "%29",
            '*' => "%2A",
            '+' => "%2B",
            ',' => "%2C",
            ';' => "%3B",
            '=' => "%3D",
            '%' => "%25",
            ' ' => "%20",
            '\n' => "%0A",
            c => {
                result.push(c);
                continue;
            }
        };

        result.push_str(replacement);
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_urldecode() {
        let url = "?foo=bar&hello=world";
        let decoded = urldecode(url);
        assert_eq!(decoded, "?foo=bar&hello=world");

        let url = "?foo=bar&hello=world%20&apples%3Doranges";
        let decoded = urldecode(url);
        assert_eq!(decoded, "?foo=bar&hello=world &apples=oranges");

        let url = "id%2Cpath%2Cmethod%2Cclient_ip";
        let decoded = urldecode(url);
        assert_eq!(decoded, "id,path,method,client_ip");

        let s = "hello&world=1234\nonetwo";
        let encoded = urlencode(s);
        let decoded = urldecode(&encoded);

        assert_eq!(decoded, s);
    }
}

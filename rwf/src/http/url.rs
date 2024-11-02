//! URL handling helpers.

/// Decode a string encoded with URL encoding.
///
/// # Arguments
///
/// * `s` - The string to decode.
///
/// # Example
///
/// ```
/// use rwf::http::urldecode;
///
/// let url = "?foo=bar&hello=world%20";
/// let decoded = urldecode(url);
/// assert_eq!(decoded, "?foo=bar&hello=world ");
/// ```
///
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
                            if (c.is_numeric() || ['A', 'B', 'C', 'D', 'E', 'F'].contains(&c)) =>
                        {
                            let _ = iter.next().unwrap();
                            num.push(c);
                        }

                        _ => {
                            let replacement = match num.as_str() {
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
        assert_eq!(decoded, "id,path,method,client_ip")
    }
}

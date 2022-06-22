//! To do.

/// To do.
pub fn normalize_identifier(value: &str) -> String {
    let mut codes = vec![];
    let mut at_start = true;
    let mut at_whitespace = true;

    // Collapse markdown whitespace and trim it.
    for char in value.chars() {
        match char {
            '\t' | '\r' | '\n' | ' ' => {
                at_whitespace = true;
            }
            _ => {
                if at_whitespace && !at_start {
                    codes.push(' ');
                }

                codes.push(char);
                at_start = false;
                at_whitespace = false;
            }
        }
    }

    // To do: test if this matches unicode.
    // Some characters are considered “uppercase”, but if their lowercase
    // counterpart is uppercased will result in a different uppercase
    // character.
    // Hence, to get that form, we perform both lower- and uppercase.
    codes
        .iter()
        .collect::<String>()
        .to_uppercase()
        .to_lowercase()
}

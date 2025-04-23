//! Deal with bytes, chars, and kinds.

use crate::util::unicode::PUNCTUATION;
use alloc::{format, string::String};
use core::str;

/// Character kinds.
#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    /// Whitespace.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a_b_ c**.
    ///    ^      ^    ^
    /// ```
    Whitespace,
    /// Punctuation.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a_b_ c**.
    ///     ^^ ^ ^    ^
    /// ```
    Punctuation,
    /// Everything else.
    ///
    /// ## Example
    ///
    /// ```markdown
    /// > | **a_b_ c**.
    ///       ^ ^  ^
    /// ```
    Other,
}

/// Get a [`char`][] right before `index` in bytes (`&[u8]`).
///
/// In most cases, markdown operates on ASCII bytes.
/// In a few cases, it is unicode aware, so we need to find an actual char.
pub fn before_index(bytes: &[u8], index: usize) -> Option<char> {
    let start = if index < 4 { 0 } else { index - 4 };
    String::from_utf8_lossy(&bytes[start..index]).chars().last()
}

/// Get a [`char`][] right at `index` in bytes (`&[u8]`).
///
/// In most cases, markdown operates on ASCII bytes.
/// In a few cases, it is unicode aware, so we need to find an actual char.
pub fn after_index(bytes: &[u8], index: usize) -> Option<char> {
    let end = if index + 4 > bytes.len() {
        bytes.len()
    } else {
        index + 4
    };
    String::from_utf8_lossy(&bytes[index..end]).chars().next()
}

/// Classify a char at `index` in bytes (`&[u8]`).
pub fn kind_after_index(bytes: &[u8], index: usize) -> Kind {
    if index == bytes.len() {
        Kind::Whitespace
    } else {
        let byte = bytes[index];
        if byte.is_ascii_whitespace() {
            Kind::Whitespace
        } else if byte.is_ascii_punctuation() {
            Kind::Punctuation
        } else if byte.is_ascii_alphanumeric() {
            Kind::Other
        } else {
            // Otherwise: seems to be an ASCII control, so it seems to be a
            // non-ASCII `char`.
            classify_opt(after_index(bytes, index))
        }
    }
}

/// Classify whether a `char` represents whitespace, punctuation, or something
/// else.
///
/// Used for attention (emphasis, strong), whose sequences can open or close
/// based on the class of surrounding characters.
///
/// ## References
///
/// * [`micromark-util-classify-character` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-util-classify-character/dev/index.js)
pub fn classify(char: char) -> Kind {
    // Unicode whitespace.
    if char.is_whitespace() {
        Kind::Whitespace
    }
    // Unicode punctuation.
    else if char.is_ascii_punctuation() || PUNCTUATION.contains(&char) {
        Kind::Punctuation
    }
    // Everything else.
    else {
        Kind::Other
    }
}

/// Like [`classify`], but supports eof as whitespace.
pub fn classify_opt(char_opt: Option<char>) -> Kind {
    char_opt.map_or(Kind::Whitespace, classify)
}

/// Format an optional `char` (`none` means eof).
pub fn format_opt(char: Option<char>) -> String {
    char.map_or("end of file".into(), |char| {
        format!("character {}", format(char))
    })
}

/// Format an optional `byte` (`none` means eof).
#[cfg(feature = "log")]
pub fn format_byte_opt(byte: Option<u8>) -> String {
    byte.map_or("end of file".into(), |byte| {
        format!("byte {}", format_byte(byte))
    })
}

/// Format a `char`.
pub fn format(char: char) -> String {
    let representation = format!("U+{:>04X}", char as u32);
    let printable = match char {
        '`' => Some("`` ` ``".into()),
        '!'..='~' => Some(format!("`{}`", char)),
        _ => None,
    };

    if let Some(char) = printable {
        format!("{} ({})", char, representation)
    } else {
        representation
    }
}

/// Format a byte (`u8`).
pub fn format_byte(byte: u8) -> String {
    let representation = format!("U+{:>04X}", byte);
    let printable = match byte {
        b'`' => Some("`` ` ``".into()),
        b'!'..=b'~' => Some(format!("`{}`", str::from_utf8(&[byte]).unwrap())),
        _ => None,
    };

    if let Some(char) = printable {
        format!("{} ({})", char, representation)
    } else {
        representation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_classify() {
        assert_eq!(
            classify(' '),
            Kind::Whitespace,
            "should classify whitespace"
        );

        assert_eq!(
            classify('.'),
            Kind::Punctuation,
            "should classify punctuation"
        );

        assert_eq!(classify('a'), Kind::Other, "should classify other");
    }

    #[test]
    fn test_format_opt() {
        assert_eq!(
            format_opt(None),
            "end of file".to_string(),
            "should format an optional char: none -> eof"
        );

        assert_eq!(
            format_opt(Some('!')),
            "character `!` (U+0021)".to_string(),
            "should format an optional char: char -> pretty"
        );
    }

    #[test]
    #[cfg(feature = "log")]
    fn test_format_byte_opt() {
        assert_eq!(
            format_byte_opt(None),
            "end of file".to_string(),
            "should format an optional byte: none -> eof"
        );

        assert_eq!(
            format_byte_opt(Some(b'!')),
            "byte `!` (U+0021)".to_string(),
            "should format an optional byte: char -> pretty"
        );
    }

    #[test]
    fn test_format() {
        assert_eq!(
            format('`'),
            "`` ` `` (U+0060)".to_string(),
            "should format a char: grave accent"
        );

        assert_eq!(
            format('!'),
            "`!` (U+0021)".to_string(),
            "should format a char: regular"
        );

        assert_eq!(
            format(' '),
            "U+0020".to_string(),
            "should format a char: unprintable"
        );
    }

    #[test]
    fn test_format_byte() {
        assert_eq!(
            format_byte(b'`'),
            "`` ` `` (U+0060)".to_string(),
            "should format a byte: grave accent"
        );

        assert_eq!(
            format_byte(b'!'),
            "`!` (U+0021)".to_string(),
            "should format a byte: regular"
        );

        assert_eq!(
            format_byte(b' '),
            "U+0020".to_string(),
            "should format a byte: unprintable"
        );
    }
}

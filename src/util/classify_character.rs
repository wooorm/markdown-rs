//! Utilities to classify characters as whitespace, punctuation, or rest.

use crate::unicode::PUNCTUATION;

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

/// Classify whether a character code represents whitespace, punctuation, or
/// something else.
///
/// Used for attention (emphasis, strong), whose sequences can open or close
/// based on the class of surrounding characters.
///
/// > 👉 **Note** that eof (`None`) is seen as whitespace.
///
/// ## References
///
/// *   [`micromark-util-classify-character` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-util-classify-character/dev/index.js)
pub fn classify(char: char) -> Kind {
    // Unicode whitespace.
    if char.is_whitespace() {
        Kind::Whitespace
    }
    // Unicode punctuation.
    else if PUNCTUATION.contains(&char) {
        Kind::Punctuation
    }
    // Everything else.
    else {
        Kind::Other
    }
}

/// Like [`classify`], but supports eof as whitespace.
pub fn classify_opt(char_opt: Option<char>) -> Kind {
    if let Some(char) = char_opt {
        classify(char)
    }
    // EOF.
    else {
        Kind::Whitespace
    }
}

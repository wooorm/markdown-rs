//! Character escapes are a construct that occurs in the string and text
//! content types.
//!
//! They’re formed with the following BNF:
//!
//! ```bnf
//! character_escape ::= '\\' ascii_punctuation
//! ```
//!
//! Like much of markdown, there are no “invalid” character escapes: just a
//! slash, or a slash followed by anything other than an ASCII punctuation
//! character, is exactly that: just a slash.
//! To escape (most) arbitrary characters, use a
//! [character reference][] instead
//! (as in, `&amp;`, `&#123;`, or say `&#x9;`).
//! It is also possible to escape a line ending in text with a similar
//! construct: a backslash followed by a line ending (that is part of the
//! construct instead of ending it).
//!
//! ## References
//!
//! *   [`character-escape.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/character-escape.js)
//! *   [*§ 2.4 Backslash escapes* in `CommonMark`](https://spec.commonmark.org/0.30/#backslash-escapes)
//!
//! [character reference]: crate::construct::character_reference
//!
//! <!-- To do: link `hard_break_escape`, `string`, `text` -->

use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

/// Start of a character escape.
///
/// ```markdown
/// a|\*b
/// a|\b
/// a|\ b
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char('\\') => {
            tokenizer.enter(TokenType::CharacterEscape);
            tokenizer.enter(TokenType::CharacterEscapeMarker);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::CharacterEscapeMarker);
            (State::Fn(Box::new(inside)), None)
        }
        _ => (State::Nok, None),
    }
}

/// Inside a character escape, after `\`.
///
/// ```markdown
/// a\|*b
/// a\|b
/// a\| b
/// ```
fn inside(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::Char(char) if char.is_ascii_punctuation() => {
            tokenizer.enter(TokenType::CharacterEscapeValue);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::CharacterEscapeValue);
            tokenizer.exit(TokenType::CharacterEscape);
            (State::Ok, None)
        }
        _ => (State::Nok, None),
    }
}

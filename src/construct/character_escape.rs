//! Character escapes are a construct that occurs in the [string][] and
//! [text][] content types.
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
//! [character reference][character_reference] instead
//! (as in, `&amp;`, `&#123;`, or say `&#x9;`).
//! It is also possible to escape a line ending in text with a similar
//! construct: a [hard break (escape)][hard_break_escape] is a backslash followed
//! by a line ending (that is part of the construct instead of ending it).
//!
//! ## Tokens
//!
//! *   [`CharacterEscape`][Token::CharacterEscape]
//! *   [`CharacterEscapeMarker`][Token::CharacterEscapeMarker]
//! *   [`CharacterEscapeValue`][Token::CharacterEscapeValue]
//!
//! ## References
//!
//! *   [`character-escape.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/character-escape.js)
//! *   [*§ 2.4 Backslash escapes* in `CommonMark`](https://spec.commonmark.org/0.30/#backslash-escapes)
//!
//! [string]: crate::content::string
//! [text]: crate::content::text
//! [character_reference]: crate::construct::character_reference
//! [hard_break_escape]: crate::construct::hard_break_escape

use crate::token::Token;
use crate::tokenizer::{State, Tokenizer};

/// Start of a character escape.
///
/// ```markdown
/// > | a\*b
///      ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some('\\') if tokenizer.parse_state.constructs.character_escape => {
            tokenizer.enter(Token::CharacterEscape);
            tokenizer.enter(Token::CharacterEscapeMarker);
            tokenizer.consume();
            tokenizer.exit(Token::CharacterEscapeMarker);
            State::Fn(Box::new(inside))
        }
        _ => State::Nok,
    }
}

/// Inside a character escape, after `\`.
///
/// ```markdown
/// > | a\*b
///       ^
/// ```
fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(char) if char.is_ascii_punctuation() => {
            tokenizer.enter(Token::CharacterEscapeValue);
            tokenizer.consume();
            tokenizer.exit(Token::CharacterEscapeValue);
            tokenizer.exit(Token::CharacterEscape);
            State::Ok
        }
        _ => State::Nok,
    }
}

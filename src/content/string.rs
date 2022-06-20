//! The string content type.
//!
//! **String** is a limited [text][] like content type which only allows
//! character escapes and character references.
//! It exists in things such as identifiers (media references, definitions),
//! titles, URLs, code (fenced) info and meta parts.
//!
//! The constructs found in string are:
//!
//! *   [Character escape][crate::construct::character_escape]
//! *   [Character reference][crate::construct::character_reference]
//!
//! [text]: crate::content::text

use crate::construct::{
    character_escape::start as character_escape, character_reference::start as character_reference,
};
use crate::tokenizer::{Code, State, StateFnResult, TokenType, Tokenizer};

// To do: line endings?

/// Before string.
///
/// ```markdown
/// |&amp;
/// |\&
/// |qwe
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt_2(character_reference, character_escape, |ok| {
            Box::new(if ok { start } else { before_data })
        })(tokenizer, code),
    }
}

/// Before string, not at a character reference or character escape.
///
/// We’re at data.
///
/// ```markdown
/// |qwe
/// ```
fn before_data(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    if let Code::None = code {
        (State::Ok, None)
    } else {
        tokenizer.enter(TokenType::Data);
        tokenizer.consume(code);
        (State::Fn(Box::new(in_data)), None)
    }
}

/// In data.
///
/// ```markdown
/// q|w|e
/// ```
fn in_data(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        // To do: line endings.
        Code::None => {
            tokenizer.exit(TokenType::Data);
            (State::Ok, None)
        }
        // To do: somehow get these markers from constructs.
        Code::Char('&' | '\\') => {
            tokenizer.exit(TokenType::Data);
            start(tokenizer, code)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(in_data)), None)
        }
    }
}

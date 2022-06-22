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
    partial_data::start as data,
};
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

const MARKERS: [Code; 2] = [
    Code::Char('&'),  // `character_reference`
    Code::Char('\\'), // `character_escape`
];

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
        _ => tokenizer.attempt_n(
            vec![Box::new(character_reference), Box::new(character_escape)],
            |ok| Box::new(if ok { start } else { before_data }),
        )(tokenizer, code),
    }
}

/// At data.
///
/// ```markdown
/// |qwe
/// ```
fn before_data(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.go(|t, c| data(t, c, MARKERS.to_vec()), start)(tokenizer, code)
}

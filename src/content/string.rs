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
    partial_data::start as data, partial_whitespace::create_resolve_whitespace,
};
use crate::tokenizer::{State, Tokenizer};

const MARKERS: [u8; 2] = [b'&', b'\\'];

/// Start of string.
pub fn start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.register_resolver(
        "whitespace".to_string(),
        Box::new(create_resolve_whitespace(false, false)),
    );
    before(tokenizer)
}

/// Before string.
fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Ok,
        _ => tokenizer.attempt_n(
            vec![Box::new(character_reference), Box::new(character_escape)],
            |ok| Box::new(if ok { before } else { before_data }),
        )(tokenizer),
    }
}

/// At data.
fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.go(|t| data(t, &MARKERS), before)(tokenizer)
}

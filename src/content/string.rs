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
    partial_data::start as data, partial_whitespace::whitespace,
};
use crate::tokenizer::{Code, State, StateFnResult, Tokenizer};

/// Before string.
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let mut markers = vec![
        Code::VirtualSpace, // `whitespace`
        Code::Char('\t'),   // `whitespace`
        Code::Char(' '),    // `hard_break_trailing`, `whitespace`
    ];

    if tokenizer.parse_state.constructs.character_reference {
        markers.push(Code::Char('&'));
    }
    if tokenizer.parse_state.constructs.character_escape {
        markers.push(Code::Char('\\'));
    }

    before_marker(tokenizer, code, markers)
}

/// Before string.
fn before_marker(tokenizer: &mut Tokenizer, code: Code, markers: Vec<Code>) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt_n(
            vec![
                Box::new(character_reference),
                Box::new(character_escape),
                Box::new(whitespace),
            ],
            |ok| {
                let func = if ok { before_marker } else { before_data };
                Box::new(move |t, c| func(t, c, markers))
            },
        )(tokenizer, code),
    }
}

/// At data.
fn before_data(tokenizer: &mut Tokenizer, code: Code, markers: Vec<Code>) -> StateFnResult {
    let copy = markers.clone();
    tokenizer.go(|t, c| data(t, c, copy), |t, c| before_marker(t, c, markers))(tokenizer, code)
}

//! The string content type.
//!
//! **String** is a limited **text** like content type which only allows
//! character escapes and character references.
//! It exists in things such as identifiers (media references, definitions),
//! titles, URLs, code (fenced) info and meta parts.
//!
//! The constructs found in strin are:
//!
//! *   [Character escape][crate::construct::character_escape]
//! *   [Character reference][crate::construct::character_reference]

use crate::construct::{
    character_escape::start as character_escape, character_reference::start as character_reference,
};
use crate::tokenizer::{Code, Event, State, StateFnResult, TokenType, Tokenizer};

/// Turn `codes` as the string content type into events.
// To do: remove this `allow` when all the content types are glued together.
#[allow(dead_code)]
pub fn string(codes: &[Code]) -> Vec<Event> {
    let mut tokenizer = Tokenizer::new();
    let (state, remainder) = tokenizer.feed(codes, Box::new(before), true);

    if let Some(ref x) = remainder {
        if !x.is_empty() {
            unreachable!("expected no final remainder {:?}", x);
        }
    }

    match state {
        State::Ok => {}
        _ => unreachable!("expected final state to be `State::Ok`"),
    }

    tokenizer.events
}

/// Before string.
///
/// First we assume character reference.
///
/// ```markdown
/// |&amp;
/// |\&
/// |qwe
/// ```
fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt(character_reference, |ok| {
            Box::new(if ok {
                before
            } else {
                before_not_character_reference
            })
        })(tokenizer, code),
    }
}

/// Before string, not at a character reference.
///
/// Assume character escape.
///
/// ```markdown
/// |\&
/// |qwe
/// ```
fn before_not_character_reference(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt(character_escape, |ok| {
            Box::new(if ok {
                before
            } else {
                before_not_character_escape
            })
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
fn before_not_character_escape(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
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
        Code::None => {
            tokenizer.exit(TokenType::Data);
            (State::Ok, None)
        }
        // To do: somehow get these markers from constructs.
        Code::Char('&' | '\\') => {
            tokenizer.exit(TokenType::Data);
            before(tokenizer, code)
        }
        _ => {
            tokenizer.consume(code);
            (State::Fn(Box::new(in_data)), None)
        }
    }
}

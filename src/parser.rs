//! Turn bytes of markdown into events.

use crate::event::{Event, Point};
use crate::state::{Name as StateName, State};
use crate::subtokenize::subtokenize;
use crate::tokenizer::Tokenizer;
use crate::{Constructs, Options};
use alloc::{string::String, vec, vec::Vec};

/// Info needed, in all content types, when parsing markdown.
///
/// Importantly, this contains a set of known definitions.
/// It also references the input value as bytes (`u8`).
#[derive(Debug)]
pub struct ParseState<'a> {
    pub constructs: &'a Constructs,
    /// List of chars.
    pub bytes: &'a [u8],
    /// Set of defined identifiers.
    pub definitions: Vec<String>,
}

/// Turn a string of markdown into events.
///
/// Passes the bytes back so the compiler can access the source.
pub fn parse<'a>(value: &'a str, options: &'a Options) -> (Vec<Event>, &'a [u8]) {
    let mut parse_state = ParseState {
        constructs: &options.constructs,
        bytes: value.as_bytes(),
        definitions: vec![],
    };

    let mut tokenizer = Tokenizer::new(
        Point {
            line: 1,
            column: 1,
            index: 0,
            vs: 0,
        },
        &parse_state,
    );

    let state = tokenizer.push(
        (0, 0),
        (parse_state.bytes.len(), 0),
        State::Next(StateName::DocumentStart),
    );
    tokenizer.flush(state, true);

    let mut events = tokenizer.events;

    parse_state.definitions = tokenizer.tokenize_state.definitions;

    while !subtokenize(&mut events, &parse_state) {}

    (events, parse_state.bytes)
}

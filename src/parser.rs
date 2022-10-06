//! Turn bytes of markdown into events.

use crate::event::{Event, Point};
use crate::state::{Name as StateName, State};
use crate::subtokenize::subtokenize;
use crate::tokenizer::Tokenizer;
use crate::ParseOptions;
use alloc::{string::String, vec, vec::Vec};

/// Info needed, in all content types, when parsing markdown.
///
/// Importantly, this contains a set of known definitions.
/// It also references the input value as bytes (`u8`).
#[derive(Debug)]
pub struct ParseState<'a> {
    /// Configuration.
    pub options: &'a ParseOptions,
    /// List of chars.
    pub bytes: &'a [u8],
    /// Set of defined definition identifiers.
    pub definitions: Vec<String>,
    /// Set of defined GFM footnote definition identifiers.
    pub gfm_footnote_definitions: Vec<String>,
}

/// Turn a string of markdown into events.
///
/// Passes the bytes back so the compiler can access the source.
pub fn parse<'a>(
    value: &'a str,
    options: &'a ParseOptions,
) -> Result<(Vec<Event>, &'a [u8]), String> {
    let mut parse_state = ParseState {
        options,
        bytes: value.as_bytes(),
        definitions: vec![],
        gfm_footnote_definitions: vec![],
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
    let mut result = tokenizer.flush(state, true)?;
    let mut events = tokenizer.events;

    parse_state
        .gfm_footnote_definitions
        .append(&mut result.gfm_footnote_definitions);
    parse_state.definitions.append(&mut result.definitions);

    loop {
        let mut result = subtokenize(&mut events, &parse_state, &None)?;
        parse_state
            .gfm_footnote_definitions
            .append(&mut result.gfm_footnote_definitions);
        parse_state.definitions.append(&mut result.definitions);

        if result.done {
            break;
        }
    }

    Ok((events, parse_state.bytes))
}

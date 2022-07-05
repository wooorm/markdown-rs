//! Turn a string of markdown into events.

use std::collections::HashSet;
// To do: this should start with `containers`, when they’re done.
use crate::content::flow::flow;
use crate::tokenizer::{Code, Event, Point};
use crate::util::codes::parse as parse_codes;

/// Information needed, in all content types, when parsing markdown.
///
/// Importantly, this contains a set of known definitions.
/// It also references the input value as [`Code`][]s.
#[derive(Debug)]
pub struct ParseState {
    /// List of codes.
    pub codes: Vec<Code>,
    /// Set of defined identifiers.
    pub definitions: HashSet<String>,
}

/// Turn a string of markdown into events.
///
/// Passes the codes back so the compiler can access the source.
pub fn parse(value: &str) -> (Vec<Event>, Vec<Code>) {
    let mut parse_state = ParseState {
        codes: parse_codes(value),
        definitions: HashSet::new(),
    };

    let events = flow(
        &mut parse_state,
        Point {
            line: 1,
            column: 1,
            offset: 0,
        },
        0,
    );

    // To do: pass whole `parse_state` back?
    (events, parse_state.codes)
}

//! Turn a string of markdown into events.

// To do: this should start with `containers`, when they’re done.
use crate::content::flow::flow;
use crate::tokenizer::{as_codes, Code, Event, Point};

pub struct ParseState {
    /// To do.
    pub codes: Vec<Code>,
    /// To do.
    pub definitions: Vec<String>,
}

/// Turn a string of markdown into events.
///
/// Passes the codes back so the compiler can access the source.
pub fn parse(value: &str) -> (Vec<Event>, Vec<Code>) {
    let parse_state = ParseState {
        codes: as_codes(value),
        definitions: vec![],
    };

    let events = flow(
        &parse_state,
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

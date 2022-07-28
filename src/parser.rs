//! Turn a string of markdown into events.

use crate::content::document::document;
use crate::tokenizer::{Event, Point};
use crate::{Constructs, Options};

/// Information needed, in all content types, when parsing markdown.
///
/// Importantly, this contains a set of known definitions.
/// It also references the input value as a `Vec<char>`.
#[derive(Debug)]
pub struct ParseState<'a> {
    pub constructs: &'a Constructs,
    /// List of chars.
    pub chars: Vec<char>,
    /// Set of defined identifiers.
    pub definitions: Vec<String>,
}

/// Turn a string of markdown into events.
///
/// Passes the codes back so the compiler can access the source.
pub fn parse<'a>(value: &str, options: &'a Options) -> (Vec<Event>, ParseState<'a>) {
    let mut parse_state = ParseState {
        constructs: &options.constructs,
        // To do: change to `u8`s?
        chars: value.chars().collect::<_>(),
        definitions: vec![],
    };

    let events = document(
        &mut parse_state,
        Point {
            line: 1,
            column: 1,
            index: 0,
            vs: 0,
        },
    );

    (events, parse_state)
}

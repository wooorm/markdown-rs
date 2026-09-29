//! Turn bytes of markdown into events.

use crate::construct::text::MARKERS as TEXT_MARKERS;
use crate::event::{Event, Point};
use crate::extension::{text_markers, token_count, TokenName};
use crate::message;
use crate::state::{Name as StateName, State};
use crate::subtokenize::subtokenize;
use crate::tokenizer::Tokenizer;
use crate::util::location::Location;
use crate::ParseOptions;
use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use core::cell::RefCell;

/// Info needed, in all content types, when parsing markdown.
///
/// Importantly, this contains a set of known definitions.
/// It also references the input value as bytes (`u8`).
#[derive(Debug)]
pub struct ParseState<'a> {
    /// Configuration.
    pub location: Option<Location>,
    /// Configuration.
    pub options: &'a ParseOptions,
    /// List of chars.
    pub bytes: &'a [u8],
    /// Set of defined definition identifiers.
    pub definitions: Vec<String>,
    /// Set of defined GFM footnote definition identifiers.
    pub gfm_footnote_definitions: Vec<String>,
    /// Bytes that can start something in text, with the markers of text
    /// constructs; empty without them.
    pub text_markers: Vec<u8>,
    /// Names of the tokens of constructs, which events refer to by index.
    pub(crate) extension_names: RefCell<Vec<TokenName>>,
    /// Pass of `subtokenize`, which is how deep in content its tokenizers
    /// are.
    pub content_depth: usize,
}

/// Turn a string of markdown into events.
///
/// Passes the bytes back so the compiler can access the source.
pub fn parse<'a>(
    value: &'a str,
    options: &'a ParseOptions,
) -> Result<(Vec<Event>, ParseState<'a>), message::Message> {
    let bytes = value.as_bytes();

    // Constructs are numbered by a `u8`.
    let constructs = options.text_constructs.len() + options.flow_constructs.len();
    if constructs > usize::from(u8::MAX) {
        return Err(message::Message {
            place: None,
            reason: format!("Unexpected {constructs} constructs, expected at most 255"),
            rule_id: Box::new("too-many-constructs".into()),
            source: Box::new("markdown-rs".into()),
        });
    }

    // Token names are numbered by a `u16`.
    let tokens = token_count(options);
    if tokens > usize::from(u16::MAX) {
        return Err(message::Message {
            place: None,
            reason: format!(
                "Unexpected {tokens} token names in constructs, expected at most 65,535"
            ),
            rule_id: Box::new("too-many-construct-tokens".into()),
            source: Box::new("markdown-rs".into()),
        });
    }

    let mut parse_state = ParseState {
        options,
        bytes,
        location: if options.mdx_esm_parse.is_some() || options.mdx_expression_parse.is_some() {
            Some(Location::new(bytes))
        } else {
            None
        },
        definitions: vec![],
        gfm_footnote_definitions: vec![],
        text_markers: text_markers(&TEXT_MARKERS, &options.text_constructs),
        extension_names: RefCell::new(vec![]),
        content_depth: 0,
    };

    let start = Point {
        line: 1,
        column: 1,
        index: 0,
        vs: 0,
    };
    let mut tokenizer = Tokenizer::new(start, &parse_state);

    let state = tokenizer.push(
        (0, 0),
        (parse_state.bytes.len(), 0),
        State::Next(StateName::DocumentStart),
    );
    let mut result = tokenizer.flush(state, true)?;
    let mut events = tokenizer.events;

    loop {
        let fn_defs = &mut parse_state.gfm_footnote_definitions;
        let defs = &mut parse_state.definitions;
        fn_defs.append(&mut result.gfm_footnote_definitions);
        defs.append(&mut result.definitions);

        if result.done {
            return Ok((events, parse_state));
        }

        parse_state.content_depth += 1;
        result = subtokenize(&mut events, &parse_state, None)?;
    }
}

//! Turn bytes of markdown into events.

use crate::construct::text::MARKERS as TEXT_MARKERS;
use crate::event::{Content, Event, Point};
use crate::extension::{bad_attention_construct, text_markers, token_count, TokenName};
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
    /// Passes of `subtokenize` so far, for documents first and then for text.
    pub content_depth: usize,
    /// Steps that constructs took, for [`assert_linear`][crate::extension::assert_linear].
    #[cfg(feature = "test-util")]
    pub(crate) extension_steps: core::cell::Cell<usize>,
    /// Steps of constructs, for [`trace`][crate::extension::trace], when on.
    #[cfg(feature = "test-util")]
    pub(crate) trace: RefCell<Option<Vec<crate::extension::TraceEvent>>>,
}

/// Turn a string of markdown into events.
///
/// Passes the bytes back so the compiler can access the source.
pub fn parse<'a>(
    value: &'a str,
    options: &'a ParseOptions,
) -> Result<(Vec<Event>, ParseState<'a>), message::Message> {
    check(options)?;
    let mut parse_state = new_state(value.as_bytes(), options);
    let events = tokenize(&mut parse_state)?;
    Ok((events, parse_state))
}

/// Check that the constructs in `options` can be numbered and placed.
fn check(options: &ParseOptions) -> Result<(), message::Message> {
    // Constructs are numbered by a `u8`.
    let constructs = options.text_constructs.len()
        + options.flow_constructs.len()
        + options.document_constructs.len();
    if constructs > usize::from(u8::MAX) {
        return Err(message::Message {
            place: None,
            reason: format!("Unexpected {constructs} constructs, expected at most 255"),
            rule_id: Box::new("too-many-constructs".into()),
            source: Box::new("markdown-rs".into()),
        });
    }

    // A delimiter run has the tokens of one, in text.
    if bad_attention_construct(options) {
        return Err(message::Message {
            place: None,
            reason: "Unexpected delimiter run with other tokens than `RunToken` or outside text, expected `RunToken` in text".into(),
            rule_id: Box::new("attention-construct".into()),
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

    Ok(())
}

/// State of a parse of `bytes`.
fn new_state<'a>(bytes: &'a [u8], options: &'a ParseOptions) -> ParseState<'a> {
    ParseState {
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
        #[cfg(feature = "test-util")]
        extension_steps: core::cell::Cell::new(0),
        #[cfg(feature = "test-util")]
        trace: RefCell::new(None),
    }
}

/// Steps that constructs take to parse `value`, whether parsing returns an
/// error or not.
#[cfg(feature = "test-util")]
pub(crate) fn extension_steps(
    value: &str,
    options: &ParseOptions,
) -> Result<usize, message::Message> {
    check(options)?;
    let mut parse_state = new_state(value.as_bytes(), options);
    let _ = tokenize(&mut parse_state);
    Ok(parse_state.extension_steps.get())
}

/// Steps of constructs while parsing `value`, whether parsing returns an
/// error or not.
#[cfg(feature = "test-util")]
pub(crate) fn trace(value: &str, options: &ParseOptions) -> Vec<crate::extension::TraceEvent> {
    if check(options).is_err() {
        return Vec::new();
    }
    let mut parse_state = new_state(value.as_bytes(), options);
    *parse_state.trace.borrow_mut() = Some(Vec::new());
    let _ = tokenize(&mut parse_state);
    let events = parse_state.trace.borrow_mut().take();
    events.unwrap_or_default()
}

/// Tokenize the document, then its content, pass after pass.
fn tokenize(parse_state: &mut ParseState) -> Result<Vec<Event>, message::Message> {
    let start = Point {
        line: 1,
        column: 1,
        index: 0,
        vs: 0,
    };
    let mut tokenizer = Tokenizer::new(start, parse_state);

    let state = tokenizer.push(
        (0, 0),
        (parse_state.bytes.len(), 0),
        State::Next(StateName::DocumentStart),
    );
    let mut result = tokenizer.flush(state, true)?;
    let mut events = tokenizer.events;

    // Documents inside constructs first, so their definitions are known in
    // all text.
    if !parse_state.options.flow_constructs.is_empty() {
        loop {
            parse_state.content_depth += 1;
            let mut nested = subtokenize(&mut events, parse_state, Some(&Content::Document))?;
            let fn_defs = &mut parse_state.gfm_footnote_definitions;
            fn_defs.append(&mut nested.gfm_footnote_definitions);
            parse_state.definitions.append(&mut nested.definitions);
            if nested.done {
                break;
            }
        }
    }

    loop {
        let fn_defs = &mut parse_state.gfm_footnote_definitions;
        let defs = &mut parse_state.definitions;
        fn_defs.append(&mut result.gfm_footnote_definitions);
        defs.append(&mut result.definitions);

        if result.done {
            return Ok(events);
        }

        parse_state.content_depth += 1;
        result = subtokenize(&mut events, parse_state, None)?;
    }
}

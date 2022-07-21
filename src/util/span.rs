//! Utilities to deal with semantic labels.

use crate::tokenizer::{Code, Event, EventType};
use crate::util::codes::serialize as serialize_codes;

/// A struct representing the span of an opening and closing event of a token.
#[derive(Debug)]
pub struct Span {
    /// Absolute offset (an `index` in `codes`) of where this span starts.
    pub start_index: usize,
    /// Absolute offset (an `index` in `codes`) of where this span ends.
    pub end_index: usize,
}

/// Get a span from an event.
///
/// Get the span of an `exit` event, by looking backwards through the events to
/// find the corresponding `enter` event.
/// This assumes that tokens with the same are not nested.
///
/// ## Panics
///
/// This function panics if an enter event is given.
/// When `micromark` is used, this function never panics.
pub fn from_exit_event(events: &[Event], index: usize) -> Span {
    let exit = &events[index];
    let end_index = exit.point.index;
    let token_type = exit.token_type.clone();
    assert_eq!(
        exit.event_type,
        EventType::Exit,
        "expected `from_exit_event` to be called on `exit` event"
    );
    let mut enter_index = index - 1;

    loop {
        let enter = &events[enter_index];
        if enter.event_type == EventType::Enter && enter.token_type == token_type {
            return Span {
                start_index: enter.point.index,
                end_index,
            };
        }

        enter_index -= 1;
    }
}

/// Serialize a span, optionally expanding tabs.
pub fn serialize(all_codes: &[Code], span: &Span, expand_tabs: bool) -> String {
    serialize_codes(codes(all_codes, span), expand_tabs)
}

/// Get a slice of codes from a span.
pub fn codes<'a>(codes: &'a [Code], span: &Span) -> &'a [Code] {
    &codes[span.start_index..span.end_index]
}

//! Utilities to deal with semantic labels.

use crate::tokenizer::{Code, Event, EventType};

/// A struct representing the span of an opening and closing event of a token.
#[derive(Debug)]
pub struct Span {
    // To do: probably needed in the future.
    // start: Point,
    /// Absolute offset (and `index` in `codes`) of where this span starts.
    pub start_index: usize,
    // To do: probably needed in the future.
    // end: Point,
    /// Absolute offset (and `index` in `codes`) of where this span ends.
    pub end_index: usize,
    // To do: probably needed in the future.
    // token_type: TokenType,
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
    // let end = exit.point.clone();
    let end_index = exit.index;
    let token_type = exit.token_type.clone();
    // To do: support `enter` events if needed and walk forwards?
    assert_eq!(
        exit.event_type,
        EventType::Exit,
        "expected `get_span` to be called on `exit` event"
    );
    let mut enter_index = index - 1;

    loop {
        let enter = &events[enter_index];
        if enter.event_type == EventType::Enter && enter.token_type == token_type {
            return Span {
                // start: enter.point.clone(),
                start_index: enter.index,
                // end,
                end_index,
                // token_type,
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

/// Serialize a slice of codes, optionally expanding tabs.
fn serialize_codes(codes: &[Code], expand_tabs: bool) -> String {
    let mut at_tab = false;
    let mut index = 0;
    let mut value: Vec<char> = vec![];

    while index < codes.len() {
        let code = codes[index];
        let mut at_tab_next = false;

        match code {
            Code::CarriageReturnLineFeed => {
                value.push('\r');
                value.push('\n');
            }
            Code::Char(char) if char == '\n' || char == '\r' => {
                value.push(char);
            }
            Code::Char(char) if char == '\t' => {
                at_tab_next = true;
                value.push(if expand_tabs { ' ' } else { char });
            }
            Code::VirtualSpace => {
                if !expand_tabs && at_tab {
                    index += 1;
                    continue;
                }
                value.push(' ');
            }
            Code::Char(char) => {
                value.push(char);
            }
            Code::None => {
                unreachable!("unexpected EOF code in codes");
            }
        }

        at_tab = at_tab_next;

        index += 1;
    }

    value.into_iter().collect()
}

//! Deal with content in other content.
//!
//! To deal with content in content, *you* (a `micromark-rs` contributor) add
//! information on events.
//! Events are a flat list, but they can be connected to each other by setting
//! `previous` and `next` links.
//! These links:
//!
//! *   …must occur on [`Enter`][EventType::Enter] events only
//! *   …must occur on void events (they are followed by their corresponding
//!     [`Exit`][EventType::Exit] event)
//! *   …must have `content_type` field to define the kind of subcontent
//!
//! Links will then be passed through a tokenizer for the corresponding content
//! type by `subtokenize`.
//! The subevents they result in are split up into slots for each linked token
//! and replace those links.
//!
//! Subevents are not immediately subtokenized again because markdown prevents
//! us from doing so due to definitions, which can occur after references, and
//! thus the whole document needs to be parsed up to the level of definitions,
//! before any level that can include references can be parsed.

use crate::content::{string::start as string, text::start as text};
use crate::parser::ParseState;
use crate::tokenizer::{ContentType, Event, EventType, State, StateFn, StateFnResult, Tokenizer};
use crate::util::{edit_map::EditMap, span};

/// Create a link between two [`Event`][]s.
///
/// Arbitrary (void) events can be linked together.
/// This optimizes for the common case where the token at `index` is connected
/// to the previous void token.
pub fn link(events: &mut [Event], index: usize) {
    link_to(events, index - 2, index);
}

/// Link two arbitrary [`Event`][]s together.
pub fn link_to(events: &mut [Event], pevious: usize, next: usize) {
    let prev = &mut events[pevious];
    assert!(
        prev.content_type.is_some(),
        "expected `content_type` on previous"
    );
    assert_eq!(prev.event_type, EventType::Enter);
    prev.next = Some(next);

    let prev_ref = &events[pevious];
    let prev_exit_ref = &events[pevious + 1];
    let curr_ref = &events[next];
    assert_eq!(prev_exit_ref.event_type, EventType::Exit);
    assert_eq!(prev_exit_ref.token_type, prev_ref.token_type);
    assert_eq!(curr_ref.content_type, prev_ref.content_type);

    let curr = &mut events[next];
    assert_eq!(curr.event_type, EventType::Enter);
    curr.previous = Some(pevious);
    // Note: the exit of this event may not exist, so don’t check for that.
}

/// Parse linked events.
///
/// Supposed to be called repeatedly, returns `1: true` when done.
pub fn subtokenize(events: Vec<Event>, parse_state: &ParseState) -> (Vec<Event>, bool) {
    let mut edit_map = EditMap::new();
    let mut done = true;
    let mut index = 0;

    while index < events.len() {
        let event = &events[index];

        // Find each first opening chunk.
        if let Some(ref content_type) = event.content_type {
            assert_eq!(event.event_type, EventType::Enter);

            // No need to enter linked events again.
            if event.previous == None {
                // Index into `events` pointing to a chunk.
                let mut link_index: Option<usize> = Some(index);
                // Subtokenizer.
                let mut tokenizer = Tokenizer::new(event.point.clone(), event.index, parse_state);
                // Substate.
                let mut result: StateFnResult = (
                    State::Fn(Box::new(if *content_type == ContentType::String {
                        string
                    } else {
                        text
                    })),
                    None,
                );

                // Loop through links to pass them in order to the subtokenizer.
                while let Some(index) = link_index {
                    let enter = &events[index];
                    assert_eq!(enter.event_type, EventType::Enter);
                    let span = span::Span {
                        start_index: enter.index,
                        end_index: events[index + 1].index,
                    };

                    if enter.previous != None {
                        tokenizer.define_skip(&enter.point, enter.index);
                    }

                    let func: Box<StateFn> = match result.0 {
                        State::Fn(func) => func,
                        _ => unreachable!("cannot be ok/nok"),
                    };

                    result = tokenizer.push(
                        span::codes(&parse_state.codes, &span),
                        func,
                        enter.next == None,
                    );
                    assert!(result.1.is_none(), "expected no remainder");
                    link_index = enter.next;
                }

                // Now, loop through all subevents to figure out which parts
                // belong where and fix deep links.
                let mut subindex = 0;
                let mut link_index = index;
                let mut slices = vec![];
                let mut slice_start = 0;

                while subindex < tokenizer.events.len() {
                    let subevent = &mut tokenizer.events[subindex];

                    // Find the first event that starts after the end we’re looking
                    // for.
                    if subevent.event_type == EventType::Enter
                        && subevent.index >= events[link_index + 1].index
                    {
                        slices.push((link_index, slice_start));
                        slice_start = subindex;
                        link_index = events[link_index].next.unwrap();
                    }

                    if subevent.content_type.is_some() {
                        // Need to call `subtokenize` again.
                        done = false;
                    }

                    // If there is a `next` link in the subevents, we have to change
                    // its index to account for the shifted events.
                    // If it points to a next event, we also change the next event’s
                    // reference back to *this* event.
                    if let Some(next) = subevent.next {
                        // The `index` in `events` where the current link is,
                        // minus 2 events (the enter and exit) for each removed
                        // link.
                        let shift = link_index - (slices.len() * 2);
                        subevent.next = Some(next + shift);
                        let next_ev = &mut tokenizer.events[next];
                        let previous = next_ev.previous.unwrap();
                        next_ev.previous = Some(previous + shift);
                    }

                    subindex += 1;
                }

                slices.push((link_index, slice_start));

                // Finally, inject the subevents.
                let mut index = slices.len();

                while index > 0 {
                    index -= 1;
                    edit_map.add(
                        slices[index].0,
                        2,
                        tokenizer.events.split_off(slices[index].1),
                    );
                }
            }
        }

        index += 1;
    }

    (edit_map.consume(events), done)
}

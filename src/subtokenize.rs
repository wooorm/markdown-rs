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

use crate::content::{flow::start as flow, string::start as string, text::start as text};
use crate::parser::ParseState;
use crate::tokenizer::{ContentType, Event, EventType, State, StateFn, StateFnResult, Tokenizer};
use crate::util::span;
use std::collections::HashMap;

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
pub fn subtokenize(mut events: Vec<Event>, parse_state: &ParseState) -> (Vec<Event>, bool) {
    let mut index = 0;
    // Map of first chunks to their tokenizer.
    let mut head_to_tokenizer: HashMap<usize, Tokenizer> = HashMap::new();
    // Map of chunks to their head and corresponding range of events.
    let mut link_to_info: HashMap<usize, (usize, usize, usize)> = HashMap::new();
    let mut done = true;

    if events.is_empty() {
        return (events, true);
    }

    while index < events.len() {
        let event = &events[index];

        // Find each first opening chunk.
        if let Some(ref content_type) = event.content_type {
            assert_eq!(event.event_type, EventType::Enter);

            // No need to enter linked events again.
            if event.previous == None {
                done = false;
                // Index into `events` pointing to a chunk.
                let mut index_opt: Option<usize> = Some(index);
                // Subtokenizer.
                let mut tokenizer = Tokenizer::new(event.point.clone(), event.index, parse_state);
                // Substate.
                let mut result: StateFnResult = (
                    State::Fn(Box::new(if *content_type == ContentType::Flow {
                        flow
                    } else if *content_type == ContentType::String {
                        string
                    } else {
                        text
                    })),
                    None,
                );
                // Indices into `codes` of each end of chunk.
                let mut ends: Vec<usize> = vec![];

                // Loop through chunks to pass them in order to the subtokenizer.
                while let Some(index_ptr) = index_opt {
                    let enter = &events[index_ptr];
                    assert_eq!(enter.event_type, EventType::Enter);
                    let span = span::Span {
                        start_index: enter.index,
                        end_index: events[index_ptr + 1].index,
                    };
                    ends.push(span.end_index);

                    if enter.previous != None {
                        tokenizer.define_skip(&enter.point);
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
                    index_opt = enter.next;
                }

                // Now, loop through all subevents (and `ends`), to figure out
                // which parts belong where.
                // Current index.
                let mut subindex = 0;
                // Index into subevents that starts the current slice.
                let mut last_start = 0;
                // Counter into `ends`: the linked token we are at.
                let mut end_index = 0;
                let mut index_opt: Option<usize> = Some(index);

                while subindex < tokenizer.events.len() {
                    let subevent = &mut tokenizer.events[subindex];

                    // Find the first event that starts after the end we’re looking
                    // for.
                    if subevent.event_type == EventType::Enter && subevent.index >= ends[end_index]
                    {
                        let link = index_opt.unwrap();
                        link_to_info.insert(link, (index, last_start, subindex));

                        last_start = subindex;
                        end_index += 1;
                        index_opt = events[link].next;
                    }

                    // If there is a `next` link in the subevents, we have to change
                    // its index to account for the shifted events.
                    // If it points to a next event, we also change the next event’s
                    // reference back to *this* event.
                    if let Some(next) = subevent.next {
                        // The `index` in `events` where the current link is,
                        // minus 2 events (the enter and exit) for each removed
                        // link.
                        let shift = index_opt.unwrap() - (end_index * 2);

                        subevent.next = Some(next + shift);
                        let next_ev = &mut tokenizer.events[next];
                        let previous = next_ev.previous.unwrap();
                        next_ev.previous = Some(previous + shift);
                    }

                    subindex += 1;
                }

                link_to_info.insert(index_opt.unwrap(), (index, last_start, subindex));
                head_to_tokenizer.insert(index, tokenizer);
            }
        }

        index += 1;
    }

    // Now that we fed everything into a tokenizer, and we know which parts
    // belong where, the final task is to splice the events from each
    // tokenizer into the current events.
    // To do: instead of splicing, it might be possible to create a new `events`
    // from each slice and slices from events?
    let mut index = events.len() - 1;

    while index > 0 {
        let slice_opt = link_to_info.get(&index);

        if let Some(slice) = slice_opt {
            let (head, start, end) = *slice;
            // If there’s a slice at this index, it must also point to a head,
            // and that head must have a tokenizer.
            let tokenizer = head_to_tokenizer.get(&head).unwrap();

            // To do: figure out a way that moves instead of clones?
            events.splice(index..(index + 2), tokenizer.events[start..end].to_vec());
        }

        index -= 1;
    }

    (events, done)
}

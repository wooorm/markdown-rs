//! Deal with content in other content.
//!
//! To deal with content in content, *you* (a `micromark-rs` contributor) add
//! information on events.
//! Events are a flat list, but they can be connected to each other by setting
//! `previous` and `next` links.
//! These links:
//!
//! *   …must occur on [`Enter`][Kind::Enter] events only
//! *   …must occur on void events (they are followed by their corresponding
//!     [`Exit`][Kind::Exit] event)
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

use crate::event::{Content, Event, Kind};
use crate::parser::ParseState;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::edit_map::EditMap;

/// Link two [`Event`][]s.
///
/// Arbitrary (void) events can be linked together.
/// This optimizes for the common case where the token at `index` is connected
/// to the previous void token.
pub fn link(events: &mut [Event], index: usize) {
    link_to(events, index - 2, index);
}

/// Link two arbitrary [`Event`][]s together.
pub fn link_to(events: &mut [Event], pevious: usize, next: usize) {
    debug_assert_eq!(events[pevious].kind, Kind::Enter);
    debug_assert_eq!(events[pevious + 1].kind, Kind::Exit);
    debug_assert_eq!(events[pevious + 1].name, events[pevious].name);
    debug_assert_eq!(events[next].kind, Kind::Enter);
    // Note: the exit of this event may not exist, so don’t check for that.

    let link_previous = events[pevious]
        .link
        .as_mut()
        .expect("expected `link` on previous");
    link_previous.next = Some(next);
    let link_next = events[next].link.as_mut().expect("expected `link` on next");
    link_next.previous = Some(pevious);

    debug_assert_eq!(
        events[pevious].link.as_ref().unwrap().content_type,
        events[next].link.as_ref().unwrap().content_type
    );
}

/// Parse linked events.
///
/// Supposed to be called repeatedly, returns `true` when done.
pub fn subtokenize(events: &mut Vec<Event>, parse_state: &ParseState) -> bool {
    let mut map = EditMap::new();
    let mut done = true;
    let mut index = 0;

    while index < events.len() {
        let event = &events[index];

        // Find each first opening chunk.
        if let Some(ref link) = event.link {
            debug_assert_eq!(event.kind, Kind::Enter);

            // No need to enter linked events again.
            if link.previous == None {
                // Index into `events` pointing to a chunk.
                let mut link_index = Some(index);
                // Subtokenizer.
                let mut tokenizer = Tokenizer::new(event.point.clone(), parse_state);
                // Substate.
                let mut state = State::Next(if link.content_type == Content::String {
                    StateName::StringStart
                } else {
                    StateName::TextStart
                });

                // Loop through links to pass them in order to the subtokenizer.
                while let Some(index) = link_index {
                    let enter = &events[index];
                    let link_curr = enter.link.as_ref().expect("expected link");
                    debug_assert_eq!(enter.kind, Kind::Enter);

                    if link_curr.previous != None {
                        tokenizer.define_skip(enter.point.clone());
                    }

                    let end = &events[index + 1].point;

                    state = tokenizer.push(
                        (enter.point.index, enter.point.vs),
                        (end.index, end.vs),
                        state,
                    );

                    link_index = link_curr.next;
                }

                tokenizer.flush(state, true);

                divide_events(&mut map, events, index, &mut tokenizer.events);

                done = false;
            }
        }

        index += 1;
    }

    map.consume(events);

    done
}

/// Divide `child_events` over links in `events`, the first of which is at
/// `link_index`.
pub fn divide_events(
    map: &mut EditMap,
    events: &[Event],
    mut link_index: usize,
    child_events: &mut Vec<Event>,
) {
    // Loop through `child_events` to figure out which parts belong where and
    // fix deep links.
    let mut child_index = 0;
    let mut slices = vec![];
    let mut slice_start = 0;
    let mut old_prev: Option<usize> = None;

    while child_index < child_events.len() {
        let current = &child_events[child_index].point;
        let end = &events[link_index + 1].point;

        // Find the first event that starts after the end we’re looking
        // for.
        if current.index > end.index || (current.index == end.index && current.vs > end.vs) {
            slices.push((link_index, slice_start));
            slice_start = child_index;
            link_index = events[link_index].link.as_ref().unwrap().next.unwrap();
        }

        // Fix sublinks.
        if let Some(sublink_curr) = &child_events[child_index].link {
            if sublink_curr.previous.is_some() {
                let old_prev = old_prev.unwrap();
                let prev_event = &mut child_events[old_prev];
                // The `index` in `events` where the current link is,
                // minus one to get the previous link,
                // minus 2 events (the enter and exit) for each removed
                // link.
                let new_link = if slices.is_empty() {
                    old_prev + link_index + 2
                } else {
                    old_prev + link_index - (slices.len() - 1) * 2
                };
                prev_event.link.as_mut().unwrap().next = Some(new_link);
            }
        }

        // If there is a `next` link in the subevents, we have to change
        // its `previous` index to account for the shifted events.
        // If it points to a next event, we also change the next event’s
        // reference back to *this* event.
        if let Some(sublink_curr) = &child_events[child_index].link {
            if let Some(next) = sublink_curr.next {
                let sublink_next = child_events[next].link.as_mut().unwrap();

                old_prev = sublink_next.previous;

                sublink_next.previous = sublink_next
                    .previous
                    // The `index` in `events` where the current link is,
                    // minus 2 events (the enter and exit) for each removed
                    // link.
                    .map(|previous| previous + link_index - (slices.len() * 2));
            }
        }

        child_index += 1;
    }

    if !child_events.is_empty() {
        slices.push((link_index, slice_start));
    }

    // Finally, inject the subevents.
    let mut index = slices.len();

    while index > 0 {
        index -= 1;
        map.add(
            slices[index].0,
            if slices[index].0 == events.len() {
                0
            } else {
                2
            },
            child_events.split_off(slices[index].1),
        );
    }
}

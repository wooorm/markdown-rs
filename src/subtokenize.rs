//! Deal with content in other content.
//!
//! To deal with content in content, *you* (a `markdown-rs` contributor) add
//! info on events.
//! Events are a flat list, but they can be connected to each other with a
//! [`Link`][crate::event::Link].
//! Links must occur on [`Enter`][Kind::Enter] events only, which are void
//! (they are followed by their corresponding [`Exit`][Kind::Exit] event).
//!
//! Links will then be passed through a tokenizer for the corresponding content
//! type by `subtokenize`.
//! The subevents they result in are split up into slots for each linked event
//! and replace those links.
//!
//! Subevents are not immediately subtokenized as markdown prevents us from
//! doing so due to definitions, which can occur after references, and thus the
//! whole document needs to be parsed up to the level of definitions, before
//! any level that can include references can be parsed.

use crate::event::{Content, Event, Kind, Name, VOID_EVENTS};
use crate::extension::ext_mut;
use crate::message;
use crate::parser::ParseState;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::{edit_map::EditMap, skip};
use alloc::{string::String, vec, vec::Vec};

#[derive(Debug)]
pub struct Subresult {
    pub done: bool,
    pub gfm_footnote_definitions: Vec<String>,
    pub definitions: Vec<String>,
}

/// Link two [`Event`][]s.
///
/// Arbitrary (void) events can be linked together.
/// This optimizes for the common case where the event at `index` is connected
/// to the previous void event.
pub fn link(events: &mut [Event], index: usize) {
    link_to(events, index - 2, index);
}

/// Link two arbitrary [`Event`][]s together.
pub fn link_to(events: &mut [Event], previous: usize, next: usize) {
    debug_assert_eq!(events[previous].kind, Kind::Enter);
    debug_assert!(
        VOID_EVENTS.iter().any(|d| d == &events[previous].name),
        "expected event to be void"
    );
    debug_assert_eq!(events[previous + 1].kind, Kind::Exit);
    debug_assert_eq!(events[previous].name, events[previous + 1].name);
    debug_assert_eq!(events[next].kind, Kind::Enter);
    debug_assert!(
        VOID_EVENTS.iter().any(|d| d == &events[next].name),
        "expected event to be void"
    );
    // Note: the exit of this event may not exist, so don’t check for that.

    let link_previous = events[previous]
        .link
        .as_mut()
        .expect("expected `link` on previous");
    link_previous.next = Some(next);
    let link_next = events[next].link.as_mut().expect("expected `link` on next");
    link_next.previous = Some(previous);

    debug_assert_eq!(
        events[previous].link.as_ref().unwrap().content,
        events[next].link.as_ref().unwrap().content,
        "expected `content` to match"
    );
}

/// Parse linked events.
///
/// Supposed to be called repeatedly, returns `true` when done.
pub fn subtokenize(
    events: &mut Vec<Event>,
    parse_state: &ParseState,
    filter: Option<&Content>,
) -> Result<Subresult, message::Message> {
    let mut map = EditMap::new();
    let mut index = 0;
    let mut value = Subresult {
        done: true,
        gfm_footnote_definitions: vec![],
        definitions: vec![],
    };
    let mut acc = (0, 0);

    while index < events.len() {
        let event = &events[index];

        // Find each first opening chunk.
        if let Some(ref link) = event.link {
            debug_assert_eq!(event.kind, Kind::Enter);

            // No need to enter linked events again.
            if link.previous.is_none()
                && (filter.is_none() || &link.content == *filter.as_ref().unwrap())
            {
                // Index into `events` pointing to a chunk.
                let mut link_index = Some(index);
                // Subtokenizer.
                let mut tokenizer = Tokenizer::new(event.point.clone(), parse_state);
                debug_assert!(
                    !matches!(link.content, Content::Flow),
                    "cannot use flow as subcontent yet"
                );
                // Substate.
                let mut state = State::Next(match link.content {
                    Content::Content => StateName::ContentDefinitionBefore,
                    Content::Document => StateName::DocumentStartNested,
                    Content::String => StateName::StringStart,
                    _ => StateName::TextStart,
                });

                // Text content of a construct.
                if event.name == Name::ExtensionChunk {
                    ext_mut(&mut tokenizer).keep_whitespace = true;
                }

                // Check if this is the first paragraph, after zero or more
                // definitions (or a blank line), in a list item.
                // Used for GFM task list items.
                if tokenizer.parse_state.options.constructs.gfm_task_list_item
                    && index > 2
                    && events[index - 1].kind == Kind::Enter
                    && events[index - 1].name == Name::Paragraph
                {
                    let before = skip::opt_back(
                        events,
                        index - 2,
                        &[
                            Name::BlankLineEnding,
                            Name::Definition,
                            Name::LineEnding,
                            Name::SpaceOrTab,
                        ],
                    );

                    if events[before].kind == Kind::Exit
                        && events[before].name == Name::ListItemPrefix
                    {
                        tokenizer
                            .tokenize_state
                            .document_at_first_paragraph_of_list_item = true;
                    }
                }

                // Loop through links to pass them in order to the subtokenizer.
                while let Some(index) = link_index {
                    let enter = &events[index];
                    let link_curr = enter.link.as_ref().expect("expected link");
                    debug_assert_eq!(enter.kind, Kind::Enter);

                    if let Some(previous) = link_curr.previous {
                        // A chunk that continues the line of the chunk before
                        // it, such as its line ending, keeps the skip of that
                        // line, whose bytes were fed.
                        if events[previous].point.line == events[previous + 1].point.line
                            && events[previous + 1].point.index == enter.point.index
                        {
                            tokenizer.define_skip_if_missing(enter.point.clone());
                        } else {
                            tokenizer.define_skip(enter.point.clone());
                        }
                    }

                    let end = &events[index + 1].point;

                    state = tokenizer.push(
                        (enter.point.index, enter.point.vs),
                        (end.index, end.vs),
                        state,
                    );

                    link_index = link_curr.next;
                }

                let mut result = tokenizer.flush(state, true)?;
                value
                    .gfm_footnote_definitions
                    .append(&mut result.gfm_footnote_definitions);
                value.definitions.append(&mut result.definitions);
                value.done = false;

                acc = divide_events(&mut map, events, index, &mut tokenizer.events, acc);
            }
        }

        index += 1;
    }

    map.consume(events);

    Ok(value)
}

/// Divide `child_events` over links in `events`, the first of which is at
/// `link_index`.
pub fn divide_events(
    map: &mut EditMap,
    events: &[Event],
    mut link_index: usize,
    child_events: &mut Vec<Event>,
    acc_before: (usize, usize),
) -> (usize, usize) {
    // Loop through `child_events` to figure out which parts belong where and
    // fix deep links.
    // Each slice replaces the enter and exit of its chunk, so a child event
    // lands at its chunk, minus 2 events for each earlier chunk, plus its
    // index in the child events.
    // Chunks may be far apart, with other events between them.
    let mut child_index = 0;
    let mut slices: Vec<(usize, usize)> = vec![];
    let mut slice_start = 0;
    let len = child_events.len();

    while child_index < len {
        let current = &child_events[child_index].point;

        // Find the chunk this event is in: a chunk can be skipped when no
        // event starts in it.
        loop {
            let end = &events[link_index + 1].point;
            if current.index < end.index || (current.index == end.index && current.vs <= end.vs) {
                break;
            }
            slices.push((link_index, slice_start));
            slice_start = child_index;
            link_index = events[link_index].link.as_ref().unwrap().next.unwrap();
        }

        // Link this event and the one before it in its chain, which is in
        // this slice or an earlier one.
        if let Some(previous) = child_events[child_index]
            .link
            .as_ref()
            .and_then(|link| link.previous)
        {
            let mut slice = slices.len();
            let (mut enter, mut start) = (link_index, slice_start);
            while previous < start {
                slice -= 1;
                enter = slices[slice].0;
                start = slices[slice].1;
            }
            // Add before subtracting: removed chunks can outnumber events.
            let new_previous = enter + previous + acc_before.1 - acc_before.0 - slice * 2;
            let new_current =
                link_index + child_index + acc_before.1 - acc_before.0 - slices.len() * 2;
            child_events[previous].link.as_mut().unwrap().next = Some(new_current);
            child_events[child_index].link.as_mut().unwrap().previous = Some(new_previous);
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
        debug_assert!(
            slices[index].0 < events.len(),
            "expected slice start in bounds"
        );
        map.add(slices[index].0, 2, child_events.split_off(slices[index].1));
    }

    (acc_before.0 + (slices.len() * 2), acc_before.1 + len)
}

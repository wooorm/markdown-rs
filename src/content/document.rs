//! The document content type.
//!
//! **Document** represents the containers, such as block quotes and lists,
//! which structure the document and contain other sections.
//!
//! The constructs found in flow are:
//!
//! *   [Block quote][crate::construct::block_quote]
//! *   [List][crate::construct::list]

use crate::parser::ParseState;
use crate::subtokenize::subtokenize;
use crate::token::Token;
use crate::tokenizer::{
    Container, ContainerState, ContentType, Event, EventType, Link, Point, State, StateName,
    Tokenizer,
};
use crate::util::{
    normalize_identifier::normalize_identifier,
    skip,
    slice::{Position, Slice},
};

/// Phases where we can exit containers.
#[derive(Debug, PartialEq)]
enum Phase {
    /// After parsing a line of lazy flow which resulted in something that
    /// exits containers before the line.
    ///
    /// ```markdown
    ///   | * a
    /// > | ```js
    ///          ^
    ///   | b
    ///   | ```
    /// ```
    After,
    /// When a new container replaces an existing container.
    ///
    /// ```markdown
    ///   | * a
    /// > | > b
    ///     ^
    /// ```
    Prefix,
    /// After everything.
    ///
    /// ```markdown
    /// > | * a
    ///        ^
    /// ```
    Eof,
}

/// Turn `codes` as the document content type into events.
pub fn document(parse_state: &mut ParseState, point: Point) -> Vec<Event> {
    let mut tokenizer = Tokenizer::new(point, parse_state);

    let state = tokenizer.push(0, parse_state.bytes.len(), StateName::DocumentStart);
    tokenizer.flush(state, true);

    let mut index = 0;
    let mut definitions = vec![];

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.event_type == EventType::Exit && event.token_type == Token::DefinitionLabelString {
            // Note: we don‘t care about virtual spaces, so `as_str` is fine.
            let id = normalize_identifier(
                Slice::from_position(
                    tokenizer.parse_state.bytes,
                    &Position::from_exit_event(&tokenizer.events, index),
                )
                .as_str(),
            );

            if !definitions.contains(&id) {
                definitions.push(id);
            }
        }

        index += 1;
    }

    let mut events = tokenizer.events;

    parse_state.definitions = definitions;

    while !subtokenize(&mut events, parse_state) {}

    events
}

/// At the beginning.
///
/// Perhaps a BOM?
///
/// ```markdown
/// > | a
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.child_tokenizer = Some(Box::new(Tokenizer::new(
        tokenizer.point.clone(),
        tokenizer.parse_state,
    )));
    tokenizer.tokenize_state.document_child_state = Some(State::Fn(StateName::FlowStart));
    tokenizer.attempt(
        StateName::BomStart,
        State::Fn(StateName::DocumentLineStart),
        State::Fn(StateName::DocumentLineStart),
    )
}

/// Start of a line.
//
/// ```markdown
/// > | * a
///     ^
/// > | > b
///     ^
/// ```
pub fn line_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.document_continued = 0;
    // Containers would only be interrupting if we’ve continued.
    tokenizer.interrupt = false;
    container_existing_before(tokenizer)
}

/// Before existing containers.
//
/// ```markdown
///   | * a
/// > | > b
///     ^
/// ```
pub fn container_existing_before(tokenizer: &mut Tokenizer) -> State {
    // If there are more existing containers, check whether the next one continues.
    if tokenizer.tokenize_state.document_continued
        < tokenizer.tokenize_state.document_container_stack.len()
    {
        let container = tokenizer
            .tokenize_state
            .document_container_stack
            .remove(tokenizer.tokenize_state.document_continued);
        let state_name = match container.kind {
            Container::BlockQuote => StateName::BlockQuoteContStart,
            Container::ListItem => StateName::ListContStart,
        };

        tokenizer.container = Some(container);
        tokenizer.attempt(
            state_name,
            State::Fn(StateName::DocumentContainerExistingAfter),
            State::Fn(StateName::DocumentContainerExistingMissing),
        )
    }
    // Otherwise, check new containers.
    else {
        container_new_before(tokenizer)
    }
}

/// At a missing, existing containers.
//
/// ```markdown
///   | * a
/// > | > b
///     ^
/// ```
pub fn container_existing_missing(tokenizer: &mut Tokenizer) -> State {
    let container = tokenizer.container.take().unwrap();
    tokenizer
        .tokenize_state
        .document_container_stack
        .insert(tokenizer.tokenize_state.document_continued, container);
    container_new_before(tokenizer)
}

/// After an existing container.
//
/// ```markdown
///   | * a
/// > |   b
///       ^
/// ```
pub fn container_existing_after(tokenizer: &mut Tokenizer) -> State {
    let container = tokenizer.container.take().unwrap();
    tokenizer
        .tokenize_state
        .document_container_stack
        .insert(tokenizer.tokenize_state.document_continued, container);
    tokenizer.tokenize_state.document_continued += 1;
    container_existing_before(tokenizer)
}

/// Before a new container.
//
/// ```markdown
/// > | * a
///     ^
/// > | > b
///     ^
/// ```
pub fn container_new_before(tokenizer: &mut Tokenizer) -> State {
    // If we have completely continued, restore the flow’s past `interrupt`
    // status.
    if tokenizer.tokenize_state.document_continued
        == tokenizer.tokenize_state.document_container_stack.len()
    {
        tokenizer.interrupt = tokenizer
            .tokenize_state
            .child_tokenizer
            .as_ref()
            .unwrap()
            .interrupt;

        // …and if we’re in a concrete construct, new containers can’t “pierce”
        // into them.
        if tokenizer
            .tokenize_state
            .child_tokenizer
            .as_ref()
            .unwrap()
            .concrete
        {
            return containers_after(tokenizer);
        }
    }

    // Check for a new container.
    // Block quote?
    tokenizer.container = Some(ContainerState {
        kind: Container::BlockQuote,
        blank_initial: false,
        size: 0,
    });

    tokenizer.attempt(
        StateName::BlockQuoteStart,
        State::Fn(StateName::DocumentContainerNewAfter),
        State::Fn(StateName::DocumentContainerNewBeforeNotBlockQuote),
    )
}

/// To do.
pub fn container_new_before_not_block_quote(tokenizer: &mut Tokenizer) -> State {
    // List item?
    tokenizer.container = Some(ContainerState {
        kind: Container::ListItem,
        blank_initial: false,
        size: 0,
    });

    tokenizer.attempt(
        StateName::ListStart,
        State::Fn(StateName::DocumentContainerNewAfter),
        State::Fn(StateName::DocumentContainersAfter),
    )
}

/// After a new container.
///
/// ```markdown
/// > | * a
///       ^
/// > | > b
///       ^
/// ```
pub fn container_new_after(tokenizer: &mut Tokenizer) -> State {
    let container = tokenizer.container.take().unwrap();

    // If we did not continue all existing containers, and there is a new one,
    // close the flow and those containers.
    if tokenizer.tokenize_state.document_continued
        != tokenizer.tokenize_state.document_container_stack.len()
    {
        exit_containers(tokenizer, &Phase::Prefix);
    }

    // Try another new container.
    tokenizer
        .tokenize_state
        .document_container_stack
        .push(container);
    tokenizer.tokenize_state.document_continued += 1;
    tokenizer.tokenize_state.document_interrupt_before = false;
    tokenizer.interrupt = false;
    container_new_before(tokenizer)
}

/// After containers, before flow.
//
/// ```markdown
/// > | * a
///       ^
/// > | > b
///       ^
/// ```
pub fn containers_after(tokenizer: &mut Tokenizer) -> State {
    if let Some(ref mut child) = tokenizer.tokenize_state.child_tokenizer {
        child.lazy = tokenizer.tokenize_state.document_continued
            != tokenizer.tokenize_state.document_container_stack.len();
        child.interrupt = tokenizer.tokenize_state.document_interrupt_before;
        child.define_skip(tokenizer.point.clone());
    }

    match tokenizer.current {
        // Note: EOL is part of data.
        None => flow_end(tokenizer),
        Some(_) => {
            let current = tokenizer.events.len();
            let previous = tokenizer.tokenize_state.document_data_index.take();
            if let Some(previous) = previous {
                tokenizer.events[previous].link.as_mut().unwrap().next = Some(current);
            }
            tokenizer.tokenize_state.document_data_index = Some(current);
            tokenizer.enter_with_link(
                Token::Data,
                Some(Link {
                    previous,
                    next: None,
                    content_type: ContentType::Flow,
                }),
            );
            flow_inside(tokenizer)
        }
    }
}

/// To do.
pub fn flow_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.exit(Token::Data);
            flow_end(tokenizer)
        }
        // Note: EOL is part of data.
        Some(b'\n') => {
            tokenizer.consume();
            tokenizer.exit(Token::Data);
            State::Fn(StateName::DocumentFlowEnd)
        }
        Some(_) => {
            tokenizer.consume();
            State::Fn(StateName::DocumentFlowInside)
        }
    }
}

/// After flow (after eol or at eof).
//
/// ```markdown
///   | * a
/// > | > b
///     ^  ^
/// ```
pub fn flow_end(tokenizer: &mut Tokenizer) -> State {
    let mut paragraph = false;
    let mut interrupt = false;

    // We have new data.
    // Note that everything except for a `null` is data.
    if tokenizer.events.len() > 1
        && tokenizer.events[tokenizer.events.len() - 1].token_type == Token::Data
    {
        let position = Position::from_exit_event(&tokenizer.events, tokenizer.events.len() - 1);

        let state = tokenizer
            .tokenize_state
            .document_child_state
            .take()
            .unwrap_or(State::Fn(StateName::FlowStart));

        let state_name = match state {
            State::Fn(state_name) => state_name,
            _ => unreachable!("expected state name"),
        };

        if let Some(ref mut child) = tokenizer.tokenize_state.child_tokenizer {
            // To do: handle VS?
            // if position.start.vs > 0 {
            // }
            let state = child.push(position.start.index, position.end.index, state_name);

            interrupt = child.interrupt;
            paragraph = matches!(state, State::Fn(StateName::ParagraphInside))
                || (!child.events.is_empty()
                    && child.events[skip::opt_back(
                        &child.events,
                        child.events.len() - 1,
                        &[Token::LineEnding],
                    )]
                    .token_type
                        == Token::Paragraph);

            tokenizer.tokenize_state.document_child_state = Some(state);

            if child.lazy && paragraph && tokenizer.tokenize_state.document_paragraph_before {
                tokenizer.tokenize_state.document_continued =
                    tokenizer.tokenize_state.document_container_stack.len();
            }

            if tokenizer.tokenize_state.document_continued
                != tokenizer.tokenize_state.document_container_stack.len()
            {
                exit_containers(tokenizer, &Phase::After);
            }
        }
    }

    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.document_continued = 0;
            exit_containers(tokenizer, &Phase::Eof);
            resolve(tokenizer);
            State::Ok
        }
        Some(_) => {
            tokenizer.tokenize_state.document_paragraph_before = paragraph;
            tokenizer.tokenize_state.document_interrupt_before = interrupt;
            line_start(tokenizer)
        }
    }
}

/// Close containers (and flow if needed).
fn exit_containers(tokenizer: &mut Tokenizer, phase: &Phase) {
    let mut stack_close = tokenizer
        .tokenize_state
        .document_container_stack
        .split_off(tokenizer.tokenize_state.document_continued);

    // So, we’re at the end of a line, but we need to close the *previous* line.
    if let Some(ref mut child) = tokenizer.tokenize_state.child_tokenizer {
        if *phase != Phase::After {
            let state = tokenizer
                .tokenize_state
                .document_child_state
                .take()
                .unwrap_or(State::Fn(StateName::FlowStart));

            child.flush(state, false);
        }

        if !stack_close.is_empty() {
            let mut inject_index = tokenizer.events.len();

            // Move past the current data to find the last container start if we’re
            // closing due to a potential lazy flow that was not lazy.
            if *phase == Phase::After {
                inject_index -= 2;
            }

            // Move past the container starts to find the last data if we’re
            // closing due to a different container or lazy flow like above.
            if *phase == Phase::After || *phase == Phase::Prefix {
                while inject_index > 0 {
                    let event = &tokenizer.events[inject_index - 1];

                    if event.token_type == Token::Data {
                        break;
                    }

                    inject_index -= 1;
                }
            }

            // Move past data starts that are just whitespace only without
            // container starts.
            while inject_index > 0 {
                let event = &tokenizer.events[inject_index - 1];

                if event.token_type == Token::Data {
                    if event.event_type == EventType::Exit {
                        let slice = Slice::from_position(
                            tokenizer.parse_state.bytes,
                            &Position::from_exit_event(&tokenizer.events, inject_index - 1),
                        );
                        let bytes = slice.bytes;
                        let mut whitespace = true;
                        let mut index = 0;
                        while index < bytes.len() {
                            match bytes[index] {
                                b'\t' | b'\n' | b'\r' | b' ' => index += 1,
                                _ => {
                                    whitespace = false;
                                    break;
                                }
                            }
                        }

                        if !whitespace {
                            break;
                        }
                    }
                } else {
                    break;
                }

                inject_index -= 1;
            }

            let ref_point = if inject_index == tokenizer.events.len() {
                tokenizer.point.clone()
            } else {
                tokenizer.events[inject_index].point.clone()
            };

            let mut exits = Vec::with_capacity(stack_close.len());

            while !stack_close.is_empty() {
                let container = stack_close.pop().unwrap();
                let token_type = match container.kind {
                    Container::BlockQuote => Token::BlockQuote,
                    Container::ListItem => Token::ListItem,
                };

                exits.push(Event {
                    event_type: EventType::Exit,
                    token_type: token_type.clone(),
                    point: ref_point.clone(),
                    link: None,
                });

                let mut stack_index = tokenizer.stack.len();
                let mut found = false;

                while stack_index > 0 {
                    stack_index -= 1;

                    if tokenizer.stack[stack_index] == token_type {
                        tokenizer.stack.remove(stack_index);
                        found = true;
                        break;
                    }
                }

                debug_assert!(found, "expected to find container token to exit");
            }

            tokenizer.map.add(inject_index, 0, exits);
        }
    }

    tokenizer.tokenize_state.document_interrupt_before = false;
}

// Inject the container events.
fn resolve(tokenizer: &mut Tokenizer) {
    let mut child = tokenizer.tokenize_state.child_tokenizer.take().unwrap();
    child.map.consume(&mut child.events);
    // To do: see if we can do this less.
    tokenizer.map.consume(&mut tokenizer.events);

    let mut link_index = skip::to(&tokenizer.events, 0, &[Token::Data]);
    // To do: share this code with `subtokenize`.
    // Now, loop through all subevents to figure out which parts
    // belong where and fix deep links.
    let mut subindex = 0;
    let mut slices = vec![];
    let mut slice_start = 0;
    let mut old_prev: Option<usize> = None;

    while subindex < child.events.len() {
        // Find the first event that starts after the end we’re looking
        // for.
        if child.events[subindex].event_type == EventType::Enter
            && child.events[subindex].point.index >= tokenizer.events[link_index + 1].point.index
        {
            slices.push((link_index, slice_start));
            slice_start = subindex;
            link_index = tokenizer.events[link_index]
                .link
                .as_ref()
                .unwrap()
                .next
                .unwrap();
        }

        // Fix sublinks.
        if let Some(sublink_curr) = &child.events[subindex].link {
            if sublink_curr.previous.is_some() {
                let old_prev = old_prev.unwrap();
                let prev_event = &mut child.events[old_prev];
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
        if let Some(sublink_curr) = &child.events[subindex].link {
            if let Some(next) = sublink_curr.next {
                let sublink_next = child.events[next].link.as_mut().unwrap();

                old_prev = sublink_next.previous;

                sublink_next.previous = sublink_next
                    .previous
                    // The `index` in `events` where the current link is,
                    // minus 2 events (the enter and exit) for each removed
                    // link.
                    .map(|previous| previous + link_index - (slices.len() * 2));
            }
        }

        subindex += 1;
    }

    if !child.events.is_empty() {
        slices.push((link_index, slice_start));
    }

    // Finally, inject the subevents.
    let mut index = slices.len();

    while index > 0 {
        index -= 1;
        let start = slices[index].0;
        tokenizer.map.add(
            start,
            if start == tokenizer.events.len() {
                0
            } else {
                2
            },
            child.events.split_off(slices[index].1),
        );
    }
    // To do: share the above code with `subtokenize`.

    let mut resolvers = child.resolvers.split_off(0);
    let mut resolver_ids = child.resolver_ids.split_off(0);
    tokenizer.resolvers.append(&mut resolvers);
    tokenizer.resolver_ids.append(&mut resolver_ids);

    // To do: see if we can do this less.
    tokenizer.map.consume(&mut tokenizer.events);

    let mut index = 0;
    let mut last_eol_enter: Option<usize> = None;
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.event_type == EventType::Exit {
            if event.token_type == Token::BlockQuote || event.token_type == Token::ListItem {
                if let Some(inject) = last_eol_enter {
                    let point = tokenizer.events[inject].point.clone();
                    let mut clone = event.clone();
                    clone.point = point;
                    // Inject a fixed exit.
                    tokenizer.map.add(inject, 0, vec![clone]);
                    // Remove this exit.
                    tokenizer.map.add(index, 1, vec![]);
                }
            } else if event.token_type == Token::LineEnding
                || event.token_type == Token::BlankLineEnding
            {
                last_eol_enter = Some(index - 1);
            } else {
                last_eol_enter = None;
            }
        }

        index += 1;
    }

    tokenizer.map.consume(&mut tokenizer.events);
}

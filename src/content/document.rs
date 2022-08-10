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
use crate::subtokenize::{divide_events, subtokenize};
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
            // Note: we don’t care about virtual spaces, so `as_str` is fine.
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
    tokenizer.tokenize_state.document_child_state = Some(State::Next(StateName::FlowStart));
    tokenizer.attempt(
        StateName::BomStart,
        State::Next(StateName::DocumentLineStart),
        State::Next(StateName::DocumentLineStart),
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
    State::Retry(StateName::DocumentContainerExistingBefore)
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
        let name = match container.kind {
            Container::BlockQuote => StateName::BlockQuoteContStart,
            Container::ListItem => StateName::ListContStart,
        };

        tokenizer.container = Some(container);
        tokenizer.attempt(
            name,
            State::Next(StateName::DocumentContainerExistingAfter),
            State::Next(StateName::DocumentContainerExistingMissing),
        )
    }
    // Otherwise, check new containers.
    else {
        State::Retry(StateName::DocumentContainerNewBefore)
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
    State::Retry(StateName::DocumentContainerNewBefore)
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
    State::Retry(StateName::DocumentContainerExistingBefore)
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
            return State::Retry(StateName::DocumentContainersAfter);
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
        State::Next(StateName::DocumentContainerNewAfter),
        State::Next(StateName::DocumentContainerNewBeforeNotBlockQuote),
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
        State::Next(StateName::DocumentContainerNewAfter),
        State::Next(StateName::DocumentContainersAfter),
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
    State::Retry(StateName::DocumentContainerNewBefore)
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
        None => State::Retry(StateName::DocumentFlowEnd),
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
            State::Retry(StateName::DocumentFlowInside)
        }
    }
}

/// To do.
pub fn flow_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.exit(Token::Data);
            State::Retry(StateName::DocumentFlowEnd)
        }
        // Note: EOL is part of data.
        Some(b'\n') => {
            tokenizer.consume();
            tokenizer.exit(Token::Data);
            State::Next(StateName::DocumentFlowEnd)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(StateName::DocumentFlowInside)
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
            .unwrap_or(State::Next(StateName::FlowStart));

        let name = match state {
            State::Next(name) => name,
            _ => unreachable!("expected state name"),
        };

        if let Some(ref mut child) = tokenizer.tokenize_state.child_tokenizer {
            // To do: handle VS?
            // if position.start.vs > 0 {
            // }
            let state = child.push(position.start.index, position.end.index, name);

            interrupt = child.interrupt;
            paragraph = matches!(state, State::Next(StateName::ParagraphInside))
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
            State::Retry(StateName::DocumentLineStart)
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
                .unwrap_or(State::Next(StateName::FlowStart));

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
    // To do: see if we can do this less.
    tokenizer.map.consume(&mut tokenizer.events);

    divide_events(
        &mut tokenizer.map,
        &tokenizer.events,
        skip::to(&tokenizer.events, 0, &[Token::Data]),
        &mut child.events,
    );

    tokenizer
        .resolvers
        .append(&mut child.resolvers.split_off(0));
    tokenizer
        .resolver_ids
        .append(&mut child.resolver_ids.split_off(0));

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

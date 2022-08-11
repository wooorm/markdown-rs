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

    let state = tokenizer.push(
        (0, 0),
        (parse_state.bytes.len(), 0),
        State::Next(StateName::DocumentStart),
    );
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
    tokenizer.tokenize_state.document_child = Some(Box::new(Tokenizer::new(
        tokenizer.point.clone(),
        tokenizer.parse_state,
    )));

    tokenizer.attempt(
        StateName::BomStart,
        State::Next(StateName::DocumentContainerExistingBefore),
        State::Next(StateName::DocumentContainerExistingBefore),
    )
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
        let container = &tokenizer.tokenize_state.document_container_stack
            [tokenizer.tokenize_state.document_continued];

        tokenizer.attempt(
            match container.kind {
                Container::BlockQuote => StateName::BlockQuoteContStart,
                Container::ListItem => StateName::ListContStart,
            },
            State::Next(StateName::DocumentContainerExistingAfter),
            State::Next(StateName::DocumentContainerNewBefore),
        )
    }
    // Otherwise, check new containers.
    else {
        State::Retry(StateName::DocumentContainerNewBefore)
    }
}

/// After an existing container.
//
/// ```markdown
///   | * a
/// > |   b
///       ^
/// ```
pub fn container_existing_after(tokenizer: &mut Tokenizer) -> State {
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
        let child = tokenizer.tokenize_state.document_child.as_ref().unwrap();

        tokenizer.interrupt = child.interrupt;

        // …and if we’re in a concrete construct, new containers can’t “pierce”
        // into them.
        if child.concrete {
            return State::Retry(StateName::DocumentContainersAfter);
        }
    }

    // Check for a new container.
    // Block quote?
    // Add a new container at the end of the stack.
    let tail = tokenizer.tokenize_state.document_container_stack.len();
    tokenizer
        .tokenize_state
        .document_container_stack
        .push(ContainerState {
            kind: Container::BlockQuote,
            blank_initial: false,
            size: 0,
        });
    // Swap the existing container with the new one.
    tokenizer
        .tokenize_state
        .document_container_stack
        .swap(tokenizer.tokenize_state.document_continued, tail);

    tokenizer.attempt(
        StateName::BlockQuoteStart,
        State::Next(StateName::DocumentContainerNewAfter),
        State::Next(StateName::DocumentContainerNewBeforeNotBlockQuote),
    )
}

/// Maybe before a new container, but not a block quote.
//
/// ```markdown
/// > | * a
///     ^
/// ```
pub fn container_new_before_not_block_quote(tokenizer: &mut Tokenizer) -> State {
    // List item?
    // We replace the empty block quote container for this new list one.
    tokenizer.tokenize_state.document_container_stack
        [tokenizer.tokenize_state.document_continued] = ContainerState {
        kind: Container::ListItem,
        blank_initial: false,
        size: 0,
    };

    tokenizer.attempt(
        StateName::ListStart,
        State::Next(StateName::DocumentContainerNewAfter),
        State::Next(StateName::DocumentContainerNewBeforeNotList),
    )
}

/// Maybe before a new container, but not a list.
//
/// ```markdown
/// > | a
///     ^
/// ```
pub fn container_new_before_not_list(tokenizer: &mut Tokenizer) -> State {
    // It wasn’t a new block quote or a list.
    // Swap the new container (in the middle) with the existing one (at the end).
    // Drop what was in the middle.
    tokenizer
        .tokenize_state
        .document_container_stack
        .swap_remove(tokenizer.tokenize_state.document_continued);

    State::Retry(StateName::DocumentContainersAfter)
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
    // It was a new block quote or a list.
    // Swap the new container (in the middle) with the existing one (at the end).
    // Take the new container.
    let container = tokenizer
        .tokenize_state
        .document_container_stack
        .swap_remove(tokenizer.tokenize_state.document_continued);

    // If we did not continue all existing containers, and there is a new one,
    // close the flow and those containers.
    if tokenizer.tokenize_state.document_continued
        != tokenizer.tokenize_state.document_container_stack.len()
    {
        exit_containers(tokenizer, &Phase::Prefix);
    }

    tokenizer
        .tokenize_state
        .document_container_stack
        .push(container);
    tokenizer.tokenize_state.document_continued += 1;
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
    let child = tokenizer.tokenize_state.document_child.as_mut().unwrap();

    child.lazy = tokenizer.tokenize_state.document_continued
        != tokenizer.tokenize_state.document_container_stack.len();
    child.define_skip(tokenizer.point.clone());

    match tokenizer.current {
        // Note: EOL is part of data.
        None => State::Retry(StateName::DocumentFlowEnd),
        Some(_) => {
            let current = tokenizer.events.len();
            let previous = tokenizer.tokenize_state.document_data_index;
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

/// In flow.
//
/// ```markdown
/// > | * ab
///       ^
/// ```
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
    let child = tokenizer.tokenize_state.document_child.as_mut().unwrap();
    let state = tokenizer
        .tokenize_state
        .document_child_state
        .unwrap_or(State::Next(StateName::FlowStart));

    tokenizer.tokenize_state.document_exits.push(None);

    let state = child.push(
        (child.point.index, child.point.vs),
        (tokenizer.point.index, tokenizer.point.vs),
        state,
    );

    let paragraph = matches!(state, State::Next(StateName::ParagraphInside))
        || (!child.events.is_empty()
            && child.events
                [skip::opt_back(&child.events, child.events.len() - 1, &[Token::LineEnding])]
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

    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.document_continued = 0;
            exit_containers(tokenizer, &Phase::Eof);
            resolve(tokenizer);
            State::Ok
        }
        Some(_) => {
            tokenizer.tokenize_state.document_continued = 0;
            tokenizer.tokenize_state.document_paragraph_before = paragraph;
            // Containers would only be interrupting if we’ve continued.
            tokenizer.interrupt = false;
            State::Retry(StateName::DocumentContainerExistingBefore)
        }
    }
}

/// Close containers (and flow if needed).
fn exit_containers(tokenizer: &mut Tokenizer, phase: &Phase) {
    let mut stack_close = tokenizer
        .tokenize_state
        .document_container_stack
        .split_off(tokenizer.tokenize_state.document_continued);

    let child = tokenizer.tokenize_state.document_child.as_mut().unwrap();

    // Flush if needed.
    if *phase != Phase::After {
        let state = tokenizer
            .tokenize_state
            .document_child_state
            .take()
            .unwrap_or(State::Next(StateName::FlowStart));

        child.flush(state, false);
    }

    if !stack_close.is_empty() {
        let index = tokenizer.tokenize_state.document_exits.len()
            - (if *phase == Phase::After { 2 } else { 1 });
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
                point: tokenizer.point.clone(),
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

        if let Some(ref mut list) = tokenizer.tokenize_state.document_exits[index] {
            list.append(&mut exits);
        } else {
            tokenizer.tokenize_state.document_exits[index] = Some(exits);
        }
    }

    child.interrupt = false;
}

// Inject everything together.
fn resolve(tokenizer: &mut Tokenizer) {
    let child = tokenizer.tokenize_state.document_child.as_mut().unwrap();

    // First, add the container exits into `child`.
    let mut child_index = 0;
    let mut line = 0;

    while child_index < child.events.len() {
        let event = &child.events[child_index];

        if event.event_type == EventType::Enter
            && (event.token_type == Token::LineEnding || event.token_type == Token::BlankLineEnding)
        {
            if let Some(mut exits) = tokenizer.tokenize_state.document_exits[line].take() {
                let mut exit_index = 0;
                while exit_index < exits.len() {
                    exits[exit_index].point = event.point.clone();
                    exit_index += 1;
                }

                child.map.add(child_index, 0, exits);
            }

            line += 1;
        }

        child_index += 1;
    }

    child.map.consume(&mut child.events);

    // Now, add all child events into our parent document tokenizer.
    divide_events(
        &mut tokenizer.map,
        &tokenizer.events,
        skip::to(&tokenizer.events, 0, &[Token::Data]),
        &mut child.events,
    );

    // Replace the flow data with actual events.
    tokenizer.map.consume(&mut tokenizer.events);

    // Now, add some final container exits due to the EOF.
    // We can’t inject them into the child earlier, as they are “outside” its
    // linked data.
    if line < tokenizer.tokenize_state.document_exits.len() {
        if let Some(mut exits) = tokenizer.tokenize_state.document_exits[line].take() {
            let mut exit_index = 0;
            while exit_index < exits.len() {
                exits[exit_index].point = tokenizer.point.clone();
                exit_index += 1;
            }

            tokenizer.events.append(&mut exits);
        }
    }

    // Add the resolvers from child.
    tokenizer
        .resolvers
        .append(&mut child.resolvers.split_off(0));
    tokenizer
        .resolver_ids
        .append(&mut child.resolver_ids.split_off(0));
}

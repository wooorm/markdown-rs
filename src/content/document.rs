//! The document content type.
//!
//! **Document** represents the containers, such as block quotes and lists,
//! which structure the document and contain other sections.
//!
//! The constructs found in flow are:
//!
//! *   [Block quote][crate::construct::block_quote]
//! *   [List][crate::construct::list]

use crate::construct::{
    block_quote::{cont as block_quote_cont, start as block_quote},
    list::{cont as list_item_const, start as list_item},
    partial_bom::start as bom,
};
use crate::content::flow::start as flow;
use crate::parser::ParseState;
use crate::subtokenize::subtokenize;
use crate::token::Token;
use crate::tokenizer::{Container, ContainerState, Event, EventType, Point, State, Tokenizer};
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

    let state = tokenizer.push(0, parse_state.bytes.len(), Box::new(start));
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
fn start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt_opt(bom, line_start)(tokenizer)
}

/// Start of a line.
//
/// ```markdown
/// > | * a
///     ^
/// > | > b
///     ^
/// ```
fn line_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.document_continued = 0;
    tokenizer.tokenize_state.document_index = tokenizer.events.len();
    tokenizer
        .tokenize_state
        .document_inject
        .push((vec![], vec![]));
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
fn container_existing_before(tokenizer: &mut Tokenizer) -> State {
    // If there are more existing containers, check whether the next one continues.
    if tokenizer.tokenize_state.document_continued
        < tokenizer.tokenize_state.document_container_stack.len()
    {
        let container = tokenizer
            .tokenize_state
            .document_container_stack
            .remove(tokenizer.tokenize_state.document_continued);
        let cont = match container.kind {
            Container::BlockQuote => block_quote_cont,
            Container::ListItem => list_item_const,
        };

        tokenizer.container = Some(container);
        tokenizer.attempt(cont, |ok| {
            Box::new(if ok {
                container_existing_after
            } else {
                container_existing_missing
            })
        })(tokenizer)
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
fn container_existing_missing(tokenizer: &mut Tokenizer) -> State {
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
fn container_existing_after(tokenizer: &mut Tokenizer) -> State {
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
fn container_new_before(tokenizer: &mut Tokenizer) -> State {
    // If we have completely continued, restore the flow’s past `interrupt`
    // status.
    if tokenizer.tokenize_state.document_continued
        == tokenizer.tokenize_state.document_container_stack.len()
    {
        tokenizer.interrupt = tokenizer.tokenize_state.document_interrupt_before;

        // …and if we’re in a concrete construct, new containers can’t “pierce”
        // into them.
        if tokenizer.concrete {
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

    tokenizer.attempt(block_quote, |ok| {
        Box::new(if ok {
            container_new_after
        } else {
            container_new_before_not_blockquote
        })
    })(tokenizer)
}

/// To do.
fn container_new_before_not_blockquote(tokenizer: &mut Tokenizer) -> State {
    // List item?
    tokenizer.container = Some(ContainerState {
        kind: Container::ListItem,
        blank_initial: false,
        size: 0,
    });

    tokenizer.attempt(list_item, |ok| {
        Box::new(if ok {
            container_new_after
        } else {
            containers_after
        })
    })(tokenizer)
}

/// After a new container.
///
/// ```markdown
/// > | * a
///       ^
/// > | > b
///       ^
/// ```
fn container_new_after(tokenizer: &mut Tokenizer) -> State {
    let container = tokenizer.container.take().unwrap();

    // Remove from the event stack.
    // We’ll properly add exits at different points manually.
    let token_type = match container.kind {
        Container::BlockQuote => Token::BlockQuote,
        Container::ListItem => Token::ListItem,
    };

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
fn containers_after(tokenizer: &mut Tokenizer) -> State {
    // Store the container events we parsed.
    tokenizer
        .tokenize_state
        .document_inject
        .last_mut()
        .unwrap()
        .0
        .append(
            &mut tokenizer
                .events
                .split_off(tokenizer.tokenize_state.document_index),
        );

    tokenizer.lazy = tokenizer.tokenize_state.document_continued
        != tokenizer.tokenize_state.document_container_stack.len();
    tokenizer.interrupt = tokenizer.tokenize_state.document_interrupt_before;
    tokenizer.define_skip_current();

    let state = tokenizer
        .tokenize_state
        .document_next
        .take()
        .unwrap_or_else(|| Box::new(flow));

    // Parse flow, pausing after eols.
    tokenizer.go_until(
        state,
        |code| matches!(code, Some(b'\n')),
        |state| Box::new(|t| flow_end(t, state)),
    )(tokenizer)
}

/// After flow (after eol or at eof).
//
/// ```markdown
///   | * a
/// > | > b
///     ^  ^
/// ```
fn flow_end(tokenizer: &mut Tokenizer, result: State) -> State {
    let paragraph = !tokenizer.events.is_empty()
        && tokenizer.events[skip::opt_back(
            &tokenizer.events,
            tokenizer.events.len() - 1,
            &[Token::LineEnding],
        )]
        .token_type
            == Token::Paragraph;

    if tokenizer.lazy && paragraph && tokenizer.tokenize_state.document_paragraph_before {
        tokenizer.tokenize_state.document_continued =
            tokenizer.tokenize_state.document_container_stack.len();
    }

    if tokenizer.tokenize_state.document_continued
        != tokenizer.tokenize_state.document_container_stack.len()
    {
        exit_containers(tokenizer, &Phase::After);
    }

    match result {
        State::Ok => {
            if !tokenizer.tokenize_state.document_container_stack.is_empty() {
                tokenizer.tokenize_state.document_continued = 0;
                exit_containers(tokenizer, &Phase::Eof);
            }

            resolve(tokenizer);
            State::Ok
        }
        State::Nok => unreachable!("unexpected `nok` from flow"),
        State::Fn(func) => {
            tokenizer.tokenize_state.document_paragraph_before = paragraph;
            tokenizer.tokenize_state.document_interrupt_before = tokenizer.interrupt;
            tokenizer.tokenize_state.document_next = Some(func);
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
    if *phase != Phase::Eof {
        tokenizer.define_skip_current();
        let mut current_events = tokenizer
            .events
            .split_off(tokenizer.tokenize_state.document_index);
        let state = tokenizer
            .tokenize_state
            .document_next
            .take()
            .unwrap_or_else(|| Box::new(flow));
        tokenizer.flush(State::Fn(state), false);

        if *phase == Phase::Prefix {
            tokenizer.tokenize_state.document_index = tokenizer.events.len();
        }

        tokenizer.events.append(&mut current_events);
    }

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
            // Note: positions are fixed later.
            point: tokenizer.point.clone(),
            link: None,
        });
    }

    let index =
        tokenizer.tokenize_state.document_inject.len() - (if *phase == Phase::Eof { 1 } else { 2 });
    tokenizer.tokenize_state.document_inject[index]
        .1
        .append(&mut exits);
    tokenizer.tokenize_state.document_interrupt_before = false;
}

// Inject the container events.
fn resolve(tokenizer: &mut Tokenizer) {
    let mut index = 0;
    let mut inject = tokenizer.tokenize_state.document_inject.split_off(0);
    inject.reverse();
    let mut first_line_ending_in_run = None;

    while let Some((before, mut after)) = inject.pop() {
        if !before.is_empty() {
            first_line_ending_in_run = None;
            tokenizer.map.add(index, 0, before);
        }

        while index < tokenizer.events.len() {
            let event = &tokenizer.events[index];

            if event.token_type == Token::LineEnding || event.token_type == Token::BlankLineEnding {
                if event.event_type == EventType::Enter {
                    first_line_ending_in_run = first_line_ending_in_run.or(Some(index));
                } else {
                    index += 1;
                    break;
                }
            } else if event.token_type == Token::SpaceOrTab {
                // Empty to allow whitespace in blank lines.
            } else if first_line_ending_in_run.is_some() {
                first_line_ending_in_run = None;
            }

            index += 1;
        }

        let point_rel = if let Some(index) = first_line_ending_in_run {
            &tokenizer.events[index].point
        } else {
            &tokenizer.point
        };

        let close_index = first_line_ending_in_run.unwrap_or(index);

        let mut subevent_index = 0;
        while subevent_index < after.len() {
            after[subevent_index].point = point_rel.clone();
            subevent_index += 1;
        }

        tokenizer.map.add(close_index, 0, after);
    }

    tokenizer.map.consume(&mut tokenizer.events);
}

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
};
use crate::content::flow::start as flow;
use crate::parser::ParseState;
use crate::subtokenize::subtokenize;
use crate::token::Token;
use crate::tokenizer::{
    Code, Container, ContainerState, Event, EventType, Point, State, StateFn, StateFnResult,
    Tokenizer,
};
use crate::util::edit_map::EditMap;
use crate::util::{
    normalize_identifier::normalize_identifier,
    skip,
    span::{from_exit_event, serialize},
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

/// State needed to parse document.
struct DocumentInfo {
    /// Number of containers that have continued.
    continued: usize,
    /// Index into `tokenizer.events` we need to track.
    index: usize,
    /// Events of containers added back later.
    inject: Vec<(Vec<Event>, Vec<Event>)>,
    /// The value of the previous line of flow’s `interrupt`.
    interrupt_before: bool,
    /// Whether the previous line of flow was a paragraph.
    paragraph_before: bool,
    /// Current containers.
    stack: Vec<ContainerState>,
    /// Current flow state function.
    next: Box<StateFn>,
}

/// Turn `codes` as the document content type into events.
pub fn document(parse_state: &mut ParseState, point: Point, index: usize) -> Vec<Event> {
    let mut tokenizer = Tokenizer::new(point, index, parse_state);

    tokenizer.push(&parse_state.codes, Box::new(start), true);

    let mut index = 0;
    let mut next_definitions = vec![];

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.event_type == EventType::Exit && event.token_type == Token::DefinitionLabelString {
            let id = normalize_identifier(
                serialize(
                    &parse_state.codes,
                    &from_exit_event(&tokenizer.events, index),
                    false,
                )
                .as_str(),
            );

            if !next_definitions.contains(&id) {
                next_definitions.push(id);
            }
        }

        index += 1;
    }

    let mut result = (tokenizer.events, false);

    parse_state.definitions = next_definitions;

    while !result.1 {
        result = subtokenize(result.0, parse_state);
    }

    result.0
}

/// Before document.
//
/// ```markdown
/// > | * a
///     ^
///   | > b
/// ```
fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let info = DocumentInfo {
        index: 0,
        continued: 0,
        inject: vec![],
        next: Box::new(flow),
        paragraph_before: false,
        interrupt_before: false,
        stack: vec![],
    };
    line_start(tokenizer, code, info)
}

/// Start of a line.
//
/// ```markdown
/// > | * a
///     ^
/// > | > b
///     ^
/// ```
fn line_start(tokenizer: &mut Tokenizer, code: Code, mut info: DocumentInfo) -> StateFnResult {
    info.index = tokenizer.events.len();
    info.inject.push((vec![], vec![]));
    info.continued = 0;
    // Containers would only be interrupting if we’ve continued.
    tokenizer.interrupt = false;
    container_existing_before(tokenizer, code, info)
}

/// Before existing containers.
//
/// ```markdown
///   | * a
/// > | > b
///     ^
/// ```
fn container_existing_before(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    // If there are more existing containers, check whether the next one continues.
    if info.continued < info.stack.len() {
        let container = info.stack.remove(info.continued);
        let cont = match container.kind {
            Container::BlockQuote => block_quote_cont,
            Container::ListItem => list_item_const,
        };

        tokenizer.container = Some(container);
        tokenizer.attempt(cont, move |ok| {
            if ok {
                Box::new(|t, c| container_existing_after(t, c, info))
            } else {
                Box::new(|t, c| container_existing_missing(t, c, info))
            }
        })(tokenizer, code)
    }
    // Otherwise, check new containers.
    else {
        container_new_before(tokenizer, code, info)
    }
}

/// At a missing, existing containers.
//
/// ```markdown
///   | * a
/// > | > b
///     ^
/// ```
fn container_existing_missing(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    let container = tokenizer.container.take().unwrap();
    info.stack.insert(info.continued, container);
    container_new_before(tokenizer, code, info)
}

/// After an existing container.
//
/// ```markdown
///   | * a
/// > |   b
///       ^
/// ```
fn container_existing_after(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    let container = tokenizer.container.take().unwrap();
    info.stack.insert(info.continued, container);
    info.continued += 1;
    container_existing_before(tokenizer, code, info)
}

/// Before a new container.
//
/// ```markdown
/// > | * a
///     ^
/// > | > b
///     ^
/// ```
fn container_new_before(
    tokenizer: &mut Tokenizer,
    code: Code,
    info: DocumentInfo,
) -> StateFnResult {
    // If we have completely continued, restore the flow’s past `interrupt`
    // status.
    if info.continued == info.stack.len() {
        tokenizer.interrupt = info.interrupt_before;

        // …and if we’re in a concrete construct, new containers can’t “pierce”
        // into them.
        if tokenizer.concrete {
            return containers_after(tokenizer, code, info);
        }
    }

    // Check for a new container.
    // Block quote?
    tokenizer.container = Some(ContainerState {
        kind: Container::BlockQuote,
        blank_initial: false,
        size: 0,
    });

    tokenizer.attempt(block_quote, move |ok| {
        if ok {
            Box::new(|t, c| container_new_after(t, c, info))
        } else {
            Box::new(|tokenizer, code| {
                // List item?
                tokenizer.container = Some(ContainerState {
                    kind: Container::ListItem,
                    blank_initial: false,
                    size: 0,
                });

                tokenizer.attempt(list_item, |ok| {
                    let func = if ok {
                        container_new_after
                    } else {
                        containers_after
                    };
                    Box::new(move |t, c| func(t, c, info))
                })(tokenizer, code)
            })
        }
    })(tokenizer, code)
}

/// After a new container.
//
/// ```markdown
/// > | * a
///       ^
/// > | > b
///       ^
/// ```
fn container_new_after(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
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

    assert!(found, "expected to find container token to exit");

    // If we did not continue all existing containers, and there is a new one,
    // close the flow and those containers.
    if info.continued != info.stack.len() {
        info = exit_containers(tokenizer, info, &Phase::Prefix);
        tokenizer.expect(code, true);
    }

    // Try another new container.
    info.stack.push(container);
    info.continued += 1;
    info.interrupt_before = false;
    tokenizer.interrupt = false;
    container_new_before(tokenizer, code, info)
}

/// After containers, before flow.
//
/// ```markdown
/// > | * a
///       ^
/// > | > b
///       ^
/// ```
fn containers_after(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    // Store the container events we parsed.
    info.inject
        .last_mut()
        .unwrap()
        .0
        .append(&mut tokenizer.events.drain(info.index..).collect::<Vec<_>>());

    tokenizer.lazy = info.continued != info.stack.len();
    tokenizer.interrupt = info.interrupt_before;
    tokenizer.define_skip(tokenizer.point.clone(), tokenizer.index);

    let state = info.next;
    info.next = Box::new(flow);

    // Parse flow, pausing after eols.
    tokenizer.go_until(
        state,
        |code| matches!(code, Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')),
        move |(state, remainder)| {
            (
                State::Fn(Box::new(move |t, c| flow_end(t, c, info, state))),
                remainder,
            )
        },
    )(tokenizer, code)
}

/// After flow (after eol or at eof).
//
/// ```markdown
///   | * a
/// > | > b
///     ^  ^
/// ```
fn flow_end(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
    result: State,
) -> StateFnResult {
    let paragraph = !tokenizer.events.is_empty()
        && tokenizer.events[skip::opt_back(
            &tokenizer.events,
            tokenizer.events.len() - 1,
            &[Token::LineEnding],
        )]
        .token_type
            == Token::Paragraph;

    if tokenizer.lazy && info.paragraph_before && paragraph {
        info.continued = info.stack.len();
    }

    if info.continued != info.stack.len() {
        info = exit_containers(tokenizer, info, &Phase::After);
        tokenizer.expect(code, true);
    }

    info.paragraph_before = paragraph;
    info.interrupt_before = tokenizer.interrupt;

    match result {
        State::Ok => {
            if !info.stack.is_empty() {
                info.continued = 0;
                info = exit_containers(tokenizer, info, &Phase::Eof);
            }

            tokenizer.events = resolve(tokenizer, &info);

            (State::Ok, Some(vec![code]))
        }
        State::Nok => unreachable!("unexpected `nok` from flow"),
        State::Fn(func) => {
            info.next = func;
            line_start(tokenizer, code, info)
        }
    }
}

/// Close containers (and flow if needed).
fn exit_containers(
    tokenizer: &mut Tokenizer,
    mut info: DocumentInfo,
    phase: &Phase,
) -> DocumentInfo {
    let mut stack_close = info.stack.drain(info.continued..).collect::<Vec<_>>();

    // So, we’re at the end of a line, but we need to close the *previous* line.
    if *phase != Phase::Eof {
        tokenizer.define_skip(tokenizer.point.clone(), tokenizer.index);
        let mut current_events = tokenizer.events.drain(info.index..).collect::<Vec<_>>();
        let next = info.next;
        info.next = Box::new(flow); // This is weird but Rust needs a function there.
        let result = tokenizer.flush(next);
        assert!(matches!(result.0, State::Ok));
        assert!(result.1.is_none());

        if *phase == Phase::Prefix {
            info.index = tokenizer.events.len();
        }

        tokenizer.events.append(&mut current_events);
    }

    let mut exits: Vec<Event> = vec![];

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
            index: tokenizer.index,
            previous: None,
            next: None,
            content_type: None,
        });
    }

    let index = info.inject.len() - (if *phase == Phase::Eof { 1 } else { 2 });
    info.inject[index].1.append(&mut exits);
    info.interrupt_before = false;

    info
}

// Inject the container events.
fn resolve(tokenizer: &mut Tokenizer, info: &DocumentInfo) -> Vec<Event> {
    let mut map = EditMap::new();
    let mut line_index = 0;
    let mut index = 0;

    let add = info.inject[line_index].0.clone();
    let mut first_line_ending_in_run: Option<usize> = None;
    map.add(0, 0, add);

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.token_type == Token::LineEnding || event.token_type == Token::BlankLineEnding {
            if event.event_type == EventType::Enter {
                first_line_ending_in_run = first_line_ending_in_run.or(Some(index));
                let mut add = info.inject[line_index].1.clone();
                let mut index = 0;
                while index < add.len() {
                    add[index].point = event.point.clone();
                    add[index].index = event.index;
                    index += 1;
                }
                if !add.is_empty() {
                    map.add(first_line_ending_in_run.unwrap(), 0, add);
                }
            } else {
                line_index += 1;
                let add = info.inject[line_index].0.clone();
                if !add.is_empty() {
                    // No longer empty.
                    first_line_ending_in_run = None;
                    map.add(index + 1, 0, add);
                }
            }
        } else if event.token_type == Token::SpaceOrTab {
            // Empty to allow whitespace in blank lines.
        } else {
            first_line_ending_in_run = None;
        }

        index += 1;
    }

    let mut add = info.inject[line_index].1.clone();
    let mut index = 0;
    while index < add.len() {
        add[index].point = tokenizer.point.clone();
        add[index].index = tokenizer.index;
        index += 1;
    }
    map.add(
        first_line_ending_in_run.unwrap_or(tokenizer.events.len()),
        0,
        add,
    );

    map.consume(&mut tokenizer.events)
}

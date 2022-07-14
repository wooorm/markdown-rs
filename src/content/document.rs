//! The document content type.
//!
//! **Document** represents the containers, such as block quotes and lists,
//! which structure the document and contain other sections.
//!
//! The constructs found in flow are:
//!
//! *   [Block quote][crate::construct::block_quote]
//! *   List

use crate::construct::{
    block_quote::{cont as block_quote_cont, end as block_quote_end, start as block_quote},
    list::{cont as list_item_const, end as list_item_end, start as list_item},
};
use crate::content::flow::start as flow;
use crate::parser::ParseState;
use crate::subtokenize::subtokenize;
use crate::token::Token;
use crate::tokenizer::{
    Code, ContainerState, Event, EventType, Point, State, StateFn, StateFnResult, Tokenizer,
};
use crate::util::edit_map::EditMap;
use crate::util::{
    normalize_identifier::normalize_identifier,
    skip,
    span::{from_exit_event, serialize},
};
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
enum Container {
    BlockQuote,
    ListItem,
}

struct DocumentInfo {
    continued: usize,
    index: usize,
    paragraph_before: bool,
    interrupt_before: bool,
    inject: Vec<(Vec<Event>, Vec<Event>)>,
    stack: Vec<Container>,
    states: Vec<ContainerState>,
    stack_close: Vec<Container>,
    next: Box<StateFn>,
}

/// Turn `codes` as the document content type into events.
pub fn document(parse_state: &mut ParseState, point: Point, index: usize) -> Vec<Event> {
    let mut tokenizer = Tokenizer::new(point, index, parse_state);

    tokenizer.push(&parse_state.codes, Box::new(start), true);

    let mut index = 0;
    let mut next_definitions: HashSet<String> = HashSet::new();

    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.event_type == EventType::Exit && event.token_type == Token::DefinitionLabelString {
            next_definitions.insert(normalize_identifier(
                serialize(
                    &parse_state.codes,
                    &from_exit_event(&tokenizer.events, index),
                    false,
                )
                .as_str(),
            ));
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

fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let info = DocumentInfo {
        index: 0,
        continued: 0,
        inject: vec![],
        next: Box::new(flow),
        paragraph_before: false,
        interrupt_before: false,
        stack: vec![],
        states: vec![],
        stack_close: vec![],
    };
    line_start(tokenizer, code, info)
}

/// Start of a new line.
fn line_start(tokenizer: &mut Tokenizer, code: Code, mut info: DocumentInfo) -> StateFnResult {
    println!("line_start");
    info.index = tokenizer.events.len();
    info.inject.push((vec![], vec![]));
    info.continued = 0;
    // Containers would only be interrupting if we’ve continued.
    tokenizer.interrupt = false;
    container_existing_before(tokenizer, code, info)
}

/// Before existing containers.
fn container_existing_before(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    println!("container_existing_before");

    // First we iterate through the open blocks, starting with the root
    // document, and descending through last children down to the last open
    // block.
    // Each block imposes a condition that the line must satisfy if the block
    // is to remain open.
    // For example, a block quote requires a `>` character.
    // A paragraph requires a non-blank line.
    // In this phase we may match all or just some of the open blocks.
    // But we cannot close unmatched blocks yet, because we may have a lazy
    // continuation line.
    if info.continued < info.stack.len() {
        let kind = &info.stack[info.continued];
        let container = info.states.remove(info.continued);
        tokenizer.container = Some(container);
        let cont = match kind {
            Container::BlockQuote => block_quote_cont,
            Container::ListItem => list_item_const,
        };
        // tokenizer.container = Some(&mut info.states[info.continued]);

        // To do: state?
        tokenizer.attempt(cont, move |ok| {
            if ok {
                Box::new(|t, c| container_existing_after(t, c, info))
            } else {
                Box::new(|t, c| container_existing_missing(t, c, info))
            }
        })(tokenizer, code)
    } else {
        // Done.
        container_new_before(tokenizer, code, info)
    }
}

fn container_existing_missing(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    let container = tokenizer.container.take().unwrap();
    info.states.insert(info.continued, container);
    container_new_before(tokenizer, code, info)
}

fn container_existing_after(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    println!("container_existing_after");
    let container = tokenizer.container.take().unwrap();
    info.states.insert(info.continued, container);
    info.continued += 1;
    container_existing_before(tokenizer, code, info)
}

fn container_new_before(
    tokenizer: &mut Tokenizer,
    code: Code,
    info: DocumentInfo,
) -> StateFnResult {
    println!("container_new_before");
    // Next, after consuming the continuation markers for existing blocks, we
    // look for new block starts (e.g. `>` for a block quote).
    // If we encounter a new block start, we close any blocks unmatched in
    // step 1 before creating the new block as a child of the last matched
    // block.
    if info.continued == info.stack.len() {
        // If we have concrete content, such as block HTML or fenced code,
        // we can’t have containers “pierce” into them, so we can immediately
        // start.
        if tokenizer.concrete {
            println!("  concrete");
            return containers_after(tokenizer, code, info);
        }

        println!(
            "  set interrupt to {:?} because we have continued (was: {:?})",
            info.interrupt_before, tokenizer.interrupt
        );
        tokenizer.interrupt = info.interrupt_before;

        //   // If we do have flow, it could still be a blank line,
        //   // but we’d be interrupting it w/ a new container if there’s a current
        //   // construct.
        //   self.interrupt = Boolean(
        //     childFlow.currentConstruct && !childFlow._gfmTableDynamicInterruptHack
        //   )
    }

    tokenizer.container = Some(ContainerState::default());
    // Check if there is a new container.
    tokenizer.attempt(block_quote, move |ok| {
        if ok {
            Box::new(|t, c| container_new_after(t, c, info, Container::BlockQuote))
        } else {
            Box::new(|tokenizer, code| {
                tokenizer.container = Some(ContainerState::default());
                tokenizer.attempt(list_item, move |ok| {
                    if ok {
                        Box::new(|t, c| container_new_after(t, c, info, Container::ListItem))
                    } else {
                        Box::new(|t, c| containers_after(t, c, info))
                    }
                })(tokenizer, code)
            })
        }
    })(tokenizer, code)
}

fn container_new_after(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
    kind: Container,
) -> StateFnResult {
    // Remove from the event stack.
    // We’ll properly add exits at different points manually.
    let end = match kind {
        Container::BlockQuote => block_quote_end,
        Container::ListItem => list_item_end,
    };

    let token_types = end();

    let mut index = 0;
    while index < token_types.len() {
        let token_type = &token_types[index];
        let mut stack_index = tokenizer.stack.len();
        let mut found = false;

        while stack_index > 0 {
            stack_index -= 1;

            if tokenizer.stack[stack_index] == *token_type {
                tokenizer.stack.remove(stack_index);
                found = true;
                break;
            }
        }

        assert!(found, "expected to find container token to exit");
        index += 1;
    }

    if info.continued < info.stack.len() {
        info.stack_close
            .append(&mut info.stack.drain(info.continued..).collect::<Vec<_>>());
        info.states.truncate(info.continued);
        info = line_end(tokenizer, info, false, true);
        tokenizer.expect(code, true);
    }

    let container = tokenizer.container.take().unwrap();
    info.states.push(container);
    info.stack.push(kind);
    info.continued = info.stack.len(); // To do: `+= 1`?
    println!(
        "  set `interrupt`, `info.interrupt_before: false` because we have new containers (before: {:?}, {:?})",
        info.interrupt_before,
        tokenizer.interrupt
    );
    info.interrupt_before = false;
    tokenizer.interrupt = info.interrupt_before;
    container_new_before(tokenizer, code, info)
}

fn containers_after(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    println!("containers_after");

    // Add all container events we parsed.
    let mut containers = tokenizer.events.drain(info.index..).collect::<Vec<_>>();
    info.inject.last_mut().unwrap().0.append(&mut containers);

    tokenizer.lazy = info.continued != info.stack.len();
    println!(
        "  restoring interrupt: {:?} (was: {:?})",
        info.interrupt_before, tokenizer.interrupt
    );
    tokenizer.interrupt = info.interrupt_before;

    // Define start.
    let point = tokenizer.point.clone();
    tokenizer.define_skip(&point);

    flow_start(tokenizer, code, info)
}

fn flow_start(tokenizer: &mut Tokenizer, code: Code, mut info: DocumentInfo) -> StateFnResult {
    println!("flow_start");

    let state = info.next;
    info.next = Box::new(flow); // This is weird but Rust needs a function there.

    tokenizer.go_until(state, eof_eol, move |(state, remainder)| {
        (
            State::Fn(Box::new(move |t, c| flow_end(t, c, info, state))),
            remainder,
        )
    })(tokenizer, code)
}

fn flow_end(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
    result: State,
) -> StateFnResult {
    println!("flow_end: lazy? {:?}", tokenizer.lazy);

    // To do: clean this!
    let index = tokenizer.events.len();
    let index = if index > 0 {
        skip::opt_back(&tokenizer.events, index - 1, &[Token::LineEnding])
    } else {
        0
    };

    let paragraph = if index > 0 {
        let ev = &tokenizer.events[index];
        ev.point.offset + 1 >= tokenizer.point.offset
            && ev.token_type == Token::Paragraph
            && !(matches!(
                tokenizer.previous,
                Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
            ) && matches!(code, Code::None))
    } else {
        false
    };

    let mut lazy = false;

    if tokenizer.lazy {
        println!("this line was lazy.");

        if info.paragraph_before && paragraph {
            println!("it was another paragraph, which is allowed.");
            lazy = true;
        } else {
            println!(
                "it was something else (prev: {:?}, cur: {:?}), which is not allowed.",
                info.paragraph_before, paragraph
            );
        }
    }

    if !lazy && info.continued < info.stack.len() {
        info.stack_close
            .append(&mut info.stack.drain(info.continued..).collect::<Vec<_>>());
        info.states.truncate(info.continued);
    }

    info = line_end(tokenizer, info, false, false);
    tokenizer.expect(code, true);

    info.paragraph_before = paragraph;
    info.interrupt_before = tokenizer.interrupt;

    match result {
        State::Ok => {
            info.stack_close
                .append(&mut info.stack.drain(..).collect::<Vec<_>>());
            info = line_end(tokenizer, info, true, false);

            let mut map = EditMap::new();
            let mut line_index = 0;
            let mut index = 0;

            println!("injections: {:#?}", info.inject);

            let add = info.inject[line_index].0.clone();
            let mut first_line_ending_in_run: Option<usize> = None;
            println!("inject:enters:0: {:?}", add.len());
            map.add(0, 0, add);

            while index < tokenizer.events.len() {
                let event = &tokenizer.events[index];

                if event.token_type == Token::LineEnding
                    || event.token_type == Token::BlankLineEnding
                {
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
                            println!(
                                "inject:exits:at-{:?}: {:?}",
                                first_line_ending_in_run,
                                add.len()
                            );
                            map.add(first_line_ending_in_run.unwrap(), 0, add);
                        }
                    } else {
                        line_index += 1;
                        let add = info.inject[line_index].0.clone();
                        if !add.is_empty() {
                            // No longer empty.
                            first_line_ending_in_run = None;
                            println!("inject:enters:at-{:?}: {:?}", index + 1, add.len());
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
            println!("inject:exits:tail-{:?}: {:?}", index, add.len());
            let mut deep_index = 0;
            while deep_index < add.len() {
                add[deep_index].point = tokenizer.point.clone();
                add[deep_index].index = tokenizer.index;
                deep_index += 1;
            }
            map.add(index, 0, add);

            tokenizer.events = map.consume(&mut tokenizer.events);
            let mut index = 0;

            println!("document:after: {:?}", tokenizer.events.len());
            while index < tokenizer.events.len() {
                let event = &tokenizer.events[index];
                println!(
                    "ev: {:?} {:?} {:?} {:?} {:?} {:?}",
                    index,
                    event.event_type,
                    event.token_type,
                    event.content_type,
                    event.previous,
                    event.next
                );
                index += 1;
            }

            (State::Ok, Some(vec![code]))
        }
        State::Nok => unreachable!("handle nok in `flow`?"),
        State::Fn(func) => {
            info.next = func;
            line_start(tokenizer, code, info)
        }
    }
}

fn line_end(
    tokenizer: &mut Tokenizer,
    mut info: DocumentInfo,
    eof: bool,
    containers_before: bool,
) -> DocumentInfo {
    let mut stack_close = info.stack_close.drain(..).collect::<Vec<_>>();
    println!("line_end: {:?}", stack_close);

    if stack_close.is_empty() {
        return info;
    }

    // So, we’re at the end of a line, but we need to close the *previous* line.
    if !eof {
        println!("closing previous flow");
        tokenizer.define_skip(&tokenizer.point.clone());
        let mut current_events = tokenizer.events.drain(info.index..).collect::<Vec<_>>();
        let next = info.next;
        info.next = Box::new(flow); // This is weird but Rust needs a function there.
        let result = tokenizer.flush(next);
        assert!(matches!(result.0, State::Ok));
        assert!(result.1.is_none());

        if containers_before {
            info.index = tokenizer.events.len();
        }

        tokenizer.events.append(&mut current_events);
    }

    let mut exits: Vec<Event> = vec![];

    while !stack_close.is_empty() {
        let kind = stack_close.pop().unwrap();
        let end = match kind {
            Container::BlockQuote => block_quote_end,
            Container::ListItem => list_item_end,
        };

        let token_types = end();

        let mut index = 0;
        while index < token_types.len() {
            let token_type = &token_types[index];

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

            index += 1;
        }
    }

    let index = info.inject.len() - (if eof { 1 } else { 2 });
    info.inject[index].1.append(&mut exits);

    println!(
        "  setting `info.interrupt_before: false` (before: {:?})",
        info.interrupt_before
    );
    info.interrupt_before = false;

    info
}

fn eof_eol(code: Code) -> bool {
    matches!(
        code,
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
    )
}

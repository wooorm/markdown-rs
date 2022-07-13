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
    list::{cont as list_const, end as list_end, start as list},
};
use crate::content::flow::start as flow;
use crate::parser::ParseState;
use crate::subtokenize::subtokenize;
use crate::token::Token;
use crate::tokenizer::{Code, Event, EventType, Point, State, StateFn, StateFnResult, Tokenizer};
use crate::util::edit_map::EditMap;
use crate::util::{
    normalize_identifier::normalize_identifier,
    skip,
    span::{from_exit_event, serialize},
};
use std::collections::HashSet;

struct DocumentInfo {
    continued: usize,
    containers_begin_index: usize,
    paragraph_before: bool,
    inject: Vec<(Vec<Event>, Vec<Event>)>,
    stack: Vec<String>,
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
        continued: 0,
        paragraph_before: false,
        inject: vec![],
        containers_begin_index: 0,
        stack: vec![],
        next: Box::new(flow),
    };
    before(tokenizer, code, info)
}

fn before(tokenizer: &mut Tokenizer, code: Code, info: DocumentInfo) -> StateFnResult {
    println!("before");
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
        let name = &info.stack[info.continued];
        let cont = if name == "blockquote" {
            block_quote_cont
        } else if name == "list" {
            list_const
        } else {
            unreachable!("todo: cont construct {:?}", name)
        };

        // To do: state?
        tokenizer.attempt(cont, move |ok| {
            if ok {
                Box::new(|t, c| document_continue(t, c, info))
            } else {
                Box::new(|t, c| check_new_containers(t, c, info))
            }
        })(tokenizer, code)
    } else {
        // Done.
        check_new_containers(tokenizer, code, info)
    }
}

fn document_continue(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    println!("document_continue");
    info.continued += 1;
    before(tokenizer, code, info)
}

fn check_new_containers(
    tokenizer: &mut Tokenizer,
    code: Code,
    info: DocumentInfo,
) -> StateFnResult {
    println!("check_new_containers");
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
            return there_is_no_new_container(tokenizer, code, info);
        }

        println!("  to do: interrupt ({:?})?", tokenizer.interrupt);
        //   // If we do have flow, it could still be a blank line,
        //   // but we’d be interrupting it w/ a new container if there’s a current
        //   // construct.
        //   self.interrupt = Boolean(
        //     childFlow.currentConstruct && !childFlow._gfmTableDynamicInterruptHack
        //   )
    } else {
        tokenizer.interrupt = false;
    }

    // Check if there is a new container.
    tokenizer.attempt(block_quote, move |ok| {
        if ok {
            Box::new(|t, c| there_is_a_new_container(t, c, info, "blockquote".to_string()))
        } else {
            Box::new(|tokenizer, code| {
                tokenizer.attempt(list, move |ok| {
                    if ok {
                        Box::new(|t, c| there_is_a_new_container(t, c, info, "list".to_string()))
                    } else {
                        Box::new(|t, c| there_is_no_new_container(t, c, info))
                    }
                })(tokenizer, code)
            })
        }
    })(tokenizer, code)
}

fn there_is_a_new_container(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
    name: String,
) -> StateFnResult {
    let size = info.continued;
    println!("exit:0: {:?}", false);
    info = exit_containers(tokenizer, info, size, false);
    tokenizer.expect(code, true);

    // Remove from the event stack.
    // We’ll properly add exits at different points manually.
    let end = if name == "blockquote" {
        block_quote_end
    } else if name == "list" {
        list_end
    } else {
        unreachable!("todo: end {:?}", name)
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

    info.stack.push(name);
    document_continue(tokenizer, code, info)
}

/// Exit open containers.
fn exit_containers(
    tokenizer: &mut Tokenizer,
    mut info: DocumentInfo,
    size: usize,
    before: bool,
) -> DocumentInfo {
    let mut exits: Vec<Event> = vec![];

    if info.stack.len() > size {
        println!("closing flow");
        let index = tokenizer.events.len();
        let result = tokenizer.flush(info.next);
        info.next = Box::new(flow); // This is weird but Rust needs a function there.
        assert!(matches!(result.0, State::Ok));
        assert!(result.1.is_none());

        let mut end = tokenizer.events.len();
        while end > 0 && end > index {
            if tokenizer.events[end - 1].token_type != Token::LineEnding {
                break;
            }

            end -= 1;
        }

        let mut add = tokenizer.events.drain(index..end).collect::<Vec<_>>();

        exits.append(&mut add);

        println!("  setting `interrupt: false`");
        tokenizer.interrupt = false;
    }

    while info.stack.len() > size {
        let name = info.stack.pop().unwrap();

        let end = if name == "blockquote" {
            block_quote_end
        } else if name == "list" {
            list_end
        } else {
            unreachable!("todo: end {:?}", name)
        };

        let token_types = end();

        let mut index = 0;
        while index < token_types.len() {
            let token_type = &token_types[index];
            println!("creating exit: {:?}", token_type);

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

    if !exits.is_empty() {
        let before = if before { 1 } else { 0 };
        let mut index = info.inject.len() - 1;
        println!("inject: {:?} {:?}", info.inject.len() - 1, before);
        if before >= index {
            // To do: maybe, if this branch happens, it’s a bug?
            println!("inject:0: {:?}", index);
            index = 0;
        } else {
            index -= before;
            println!("set: {:?}", index);
        }
        info.inject[index].1.append(&mut exits);
    }

    info
}

fn there_is_no_new_container(
    tokenizer: &mut Tokenizer,
    code: Code,
    info: DocumentInfo,
) -> StateFnResult {
    println!("there_is_no_new_container");
    tokenizer.lazy = info.continued != info.stack.len();
    // lineStartOffset = self.now().offset
    flow_start(tokenizer, code, info)
}

fn flow_start(tokenizer: &mut Tokenizer, code: Code, mut info: DocumentInfo) -> StateFnResult {
    println!("flow_start");

    let containers = tokenizer
        .events
        .drain(info.containers_begin_index..)
        .collect::<Vec<_>>();

    info.inject.push((containers, vec![]));

    // Define start.
    let point = tokenizer.point.clone();
    tokenizer.define_skip(&point);

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

    let mut continued = info.continued;
    let size = info.stack.len();

    if tokenizer.lazy {
        println!("this line was lazy.");

        if info.paragraph_before && paragraph {
            println!("it was another paragraph, which is allowed.");
            continued = size;
        } else {
            println!(
                "it was something else (prev: {:?}, cur: {:?}), which is not allowed.",
                info.paragraph_before, paragraph
            );
        }
    }

    // Exit containers.
    println!("exit:1: {:?}", true);
    info = exit_containers(tokenizer, info, continued, true);
    tokenizer.expect(code, true);

    info.continued = 0;
    info.paragraph_before = paragraph;
    info.containers_begin_index = tokenizer.events.len();

    match result {
        State::Ok => {
            println!("exit:3: {:?}", false);
            info = exit_containers(tokenizer, info, 0, false);
            tokenizer.expect(code, true);

            let mut map = EditMap::new();
            let mut line_index = 0;
            let mut index = 0;

            let add = info.inject[line_index].0.clone();
            let mut first_line_ending_in_run: Option<usize> = None;
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
                            map.add(first_line_ending_in_run.unwrap(), 0, add);
                        }
                    } else {
                        line_index += 1;
                        let add = info.inject[line_index].0.clone();
                        if !add.is_empty() {
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
            before(tokenizer, code, info)
        }
    }
}

fn eof_eol(code: Code) -> bool {
    matches!(
        code,
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r')
    )
}

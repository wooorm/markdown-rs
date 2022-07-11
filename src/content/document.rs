//! The document content type.
//!
//! **Document** represents the containers, such as block quotes and lists,
//! which structure the document and contain other sections.
//!
//! The constructs found in flow are:
//!
//! *   [Block quote][crate::construct::block_quote]
//! *   List

use crate::construct::block_quote::{
    cont as block_quote_cont, end as block_quote_end, start as block_quote,
};
use crate::content::flow::start as flow;
use crate::parser::ParseState;
use crate::subtokenize::subtokenize;
use crate::token::Token;
use crate::tokenizer::{Code, Event, EventType, Point, State, StateFn, StateFnResult, Tokenizer};
use crate::util::edit_map::EditMap;
use crate::util::{
    normalize_identifier::normalize_identifier,
    span::{from_exit_event, serialize},
};
use std::collections::HashSet;

struct DocumentInfo {
    continued: usize,
    containers_begin_index: usize,
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
        inject: vec![],
        containers_begin_index: 0,
        stack: vec![],
        next: Box::new(flow),
    };
    before(tokenizer, code, info)
}

fn before(tokenizer: &mut Tokenizer, code: Code, info: DocumentInfo) -> StateFnResult {
    println!("before: check existing open containers");
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
        // To do: list.
        let cont = if name == "blockquote" {
            block_quote_cont
        } else {
            unreachable!("todo: cont construct {:?}", name)
        };

        // To do: state?
        println!("check existing: {:?}", name);

        tokenizer.attempt(cont, move |ok| {
            if ok {
                Box::new(|t, c| document_continue(t, c, info))
            } else {
                Box::new(|t, c| check_new_containers(t, c, info))
            }
        })(tokenizer, code)
    } else {
        // Done.
        println!("check new:");
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

    println!("  to do: close flow sometimes?");
    // // Note: this field is called `_closeFlow` but it also closes containers.
    // // Perhaps a good idea to rename it but it’s already used in the wild by
    // // extensions.
    // if (self.containerState._closeFlow) {
    //   self.containerState._closeFlow = undefined

    //   if (childFlow) {
    //     closeFlow()
    //   }

    //   // Note: this algorithm for moving events around is similar to the
    //   // algorithm when dealing with lazy lines in `writeToChild`.
    //   const indexBeforeExits = self.events.length
    //   let indexBeforeFlow = indexBeforeExits
    //   /** @type {Point|undefined} */
    //   let point

    //   // Find the flow chunk.
    //   while (indexBeforeFlow--) {
    //     if (
    //       self.events[indexBeforeFlow][0] === 'exit' &&
    //       self.events[indexBeforeFlow][1].type === types.chunkFlow
    //     ) {
    //       point = self.events[indexBeforeFlow][1].end
    //       break
    //     }
    //   }

    //   assert(point, 'could not find previous flow chunk')

    let size = info.continued;
    info = exit_containers(tokenizer, info, size, true);
    tokenizer.expect(code, true);

    //   // Fix positions.
    //   let index = indexBeforeExits

    //   while (index < self.events.length) {
    //     self.events[index][1].end = Object.assign({}, point)
    //     index++
    //   }

    //   // Inject the exits earlier (they’re still also at the end).
    //   splice(
    //     self.events,
    //     indexBeforeFlow + 1,
    //     0,
    //     self.events.slice(indexBeforeExits)
    //   )

    //   // Discard the duplicate exits.
    //   self.events.length = index

    //   return checkNewContainers(code)
    // }

    before(tokenizer, code, info)
}
// documentContinue

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
        //   // No need to `check` whether there’s a container, if `exitContainers`
        //   // would be moot.
        //   // We can instead immediately `attempt` to parse one.
        //   if (!childFlow) {
        //     return documentContinued(code)
        //   }

        // If we have concrete content, such as block HTML or fenced code,
        // we can’t have containers “pierce” into them, so we can immediately
        // start.
        if tokenizer.concrete {
            println!("  concrete!");
            return there_is_no_new_container(tokenizer, code, info);
        }

        println!("  to do: interrupt ({:?})?", tokenizer.interrupt);
        //   // If we do have flow, it could still be a blank line,
        //   // but we’d be interrupting it w/ a new container if there’s a current
        //   // construct.
        //   self.interrupt = Boolean(
        //     childFlow.currentConstruct && !childFlow._gfmTableDynamicInterruptHack
        //   )
    }

    // Check if there is a new container.
    // To do: list.
    tokenizer.attempt(block_quote, move |ok| {
        if ok {
            Box::new(|t, c| there_is_a_new_container(t, c, info, "blockquote".to_string()))
        } else {
            Box::new(|t, c| there_is_no_new_container(t, c, info))
        }
    })(tokenizer, code)
}

fn there_is_a_new_container(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
    name: String,
) -> StateFnResult {
    println!("there_is_a_new_container");
    let size = info.continued;
    info = exit_containers(tokenizer, info, size, true);
    tokenizer.expect(code, true);

    // Remove from the event stack.
    // We’ll properly add exits at different points manually.
    // To do: list.
    let end = if name == "blockquote" {
        block_quote_end
    } else {
        unreachable!("todo: cont {:?}", name)
    };

    println!("creating exit (a) for `{:?}`", name);

    let token_types = end();

    let mut index = 0;
    while index < token_types.len() {
        let token_type = &token_types[index];
        let mut stack_index = tokenizer.stack.len();
        println!("stack: {:?}", tokenizer.stack);
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

    println!("add to stack: {:?}", name);
    info.stack.push(name);

    info.continued += 1;
    document_continued(tokenizer, code, info)
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
        // To do: inject these somewhere? Fix positions?
        println!("closing flow. To do: are these resulting exits okay?");
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

        println!("evs: {:#?}", add);
        exits.append(&mut add);

        println!("  setting `interrupt: false`");
        tokenizer.interrupt = false;
    }

    while info.stack.len() > size {
        let name = info.stack.pop().unwrap();

        // To do: list.
        let end = if name == "blockquote" {
            block_quote_end
        } else {
            unreachable!("todo: cont {:?}", name)
        };

        println!("creating exit (b) for `{:?}`", name);

        let token_types = end();

        let mut index = 0;
        while index < token_types.len() {
            let token_type = &token_types[index];

            exits.push(Event {
                event_type: EventType::Exit,
                token_type: token_type.clone(),
                // To do: fix position later.
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
        let index = info.inject.len() - 1 - (if before { 1 } else { 0 });
        info.inject[index].1.append(&mut exits);
    }

    // println!("exits: {:?} {:?}", info.inject, exits);

    info
}

fn there_is_no_new_container(
    tokenizer: &mut Tokenizer,
    code: Code,
    info: DocumentInfo,
) -> StateFnResult {
    let lazy = info.continued != info.stack.len();
    tokenizer.lazy = lazy;
    println!("there is no new container");
    if lazy {
        println!(
            "  This line will be lazy. Depending on what is parsed now, we need to close containers before?"
        );
    }
    // lineStartOffset = self.now().offset
    flow_start(tokenizer, code, info)
}

fn document_continued(tokenizer: &mut Tokenizer, code: Code, info: DocumentInfo) -> StateFnResult {
    println!("document_continued");

    // Try new containers.
    // To do: list.
    tokenizer.attempt(block_quote, |ok| {
        if ok {
            Box::new(|t, c| container_continue(t, c, info))
        } else {
            Box::new(|t, c| {
                // To do: this looks like a bug?
                t.lazy = false;
                flow_start(t, c, info)
            })
        }
    })(tokenizer, code)
}

fn container_continue(
    tokenizer: &mut Tokenizer,
    code: Code,
    mut info: DocumentInfo,
) -> StateFnResult {
    println!("container_continue");
    // assert(
    //   self.currentConstruct,
    //   'expected `currentConstruct` to be defined on tokenizer'
    // )
    // assert(
    //   self.containerState,
    //   'expected `containerState` to be defined on tokenizer'
    // )
    info.continued += 1;
    // To do: add to stack?
    // stack.push([self.currentConstruct, self.containerState])
    // Try another.
    document_continued(tokenizer, code, info)
}

fn flow_start(tokenizer: &mut Tokenizer, code: Code, mut info: DocumentInfo) -> StateFnResult {
    let containers = tokenizer
        .events
        .drain(info.containers_begin_index..)
        .collect::<Vec<_>>();

    info.inject.push((containers, vec![]));

    // Exit containers.
    let size = info.continued;
    info = exit_containers(tokenizer, info, size, true);
    tokenizer.expect(code, true);

    // Define start.
    let point = tokenizer.point.clone();
    tokenizer.define_skip(&point);

    let state = info.next;
    info.next = Box::new(flow); // This is weird but Rust needs a function there.

    println!("flow_start:before");
    tokenizer.go_until(state, eof_eol, move |(state, remainder)| {
        println!("flow_start:after");
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
    println!("flow_end");
    let was_lazy = tokenizer.lazy;

    if was_lazy {
        println!(
            "this line was lazy. Depeding on what was parsed, we need to exit containers after it?"
        );
    }

    info.continued = 0;
    info.containers_begin_index = tokenizer.events.len();

    match result {
        State::Ok => {
            println!("State::Ok");
            info = exit_containers(tokenizer, info, 0, false);
            tokenizer.expect(code, true);
            // println!("document:inject: {:?}", info.inject);

            let mut map = EditMap::new();
            let mut line_index = 0;
            let mut index = 0;

            let add = info.inject[line_index].0.clone();
            println!("add enters at start: {:?}", add);
            map.add(0, 0, add);

            while index < tokenizer.events.len() {
                let event = &tokenizer.events[index];

                if event.token_type == Token::LineEnding
                    || event.token_type == Token::BlankLineEnding
                {
                    println!("eol: {:?}", event.point);
                    if event.event_type == EventType::Enter {
                        let mut add = info.inject[line_index].1.clone();
                        let mut deep_index = 0;
                        while deep_index < add.len() {
                            add[deep_index].point = event.point.clone();
                            add[deep_index].index = event.index;
                            deep_index += 1;
                        }
                        println!("add exits before: {:?}", add);
                        map.add(index, 0, add);
                    } else {
                        line_index += 1;
                        let add = info.inject[line_index].0.clone();
                        println!("add enters after: {:?}", add);
                        map.add(index + 1, 0, add);
                    }
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
            println!("add exits at end: {:?}", add);
            map.add(index, 0, add);

            tokenizer.events = map.consume(&mut tokenizer.events);
            let mut index = 0;
            println!("document:inject:ends: {:?}", tokenizer.events.len());
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

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
    stack: Vec<String>,
    next: Box<StateFn>,
    last_line_ending_index: Option<usize>,
    map: EditMap,
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
        stack: vec![],
        next: Box::new(flow),
        last_line_ending_index: None,
        map: EditMap::new(),
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
    exit_containers(tokenizer, &mut info, size);

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
        println!("  to do: interrupt ({:?})?", tokenizer.interrupt);
        //   // No need to `check` whether there’s a container, of `exitContainers`
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
            return flow_start(tokenizer, code, info);
        }

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
    println!("  todo: close_flow");
    // if (childFlow) closeFlow()
    let size = info.continued;
    exit_containers(tokenizer, &mut info, size);
    info.stack.push(name);
    info.continued += 1;
    document_continued(tokenizer, code, info)
}

/// Exit open containers.
fn exit_containers(tokenizer: &mut Tokenizer, info: &mut DocumentInfo, size: usize) {
    while info.stack.len() > size {
        let name = info.stack.pop().unwrap();

        // To do: list.
        let end = if name == "blockquote" {
            block_quote_end
        } else {
            unreachable!("todo: cont {:?}", name)
        };

        // To do: improve below code.
        let insert_index = if let Some(index) = info.last_line_ending_index {
            index
        } else {
            tokenizer.events.len()
        };
        let eol_point = if let Some(index) = info.last_line_ending_index {
            tokenizer.events[index].point.clone()
        } else {
            tokenizer.point.clone()
        };
        let eol_index = if let Some(index) = info.last_line_ending_index {
            tokenizer.events[index].index
        } else {
            tokenizer.index
        };

        let token_types = end();

        let mut index = 0;
        while index < token_types.len() {
            let token_type = &token_types[index];

            info.map.add(
                insert_index,
                0,
                vec![Event {
                    event_type: EventType::Exit,
                    token_type: token_type.clone(),
                    point: eol_point.clone(),
                    index: eol_index,
                    previous: None,
                    next: None,
                    content_type: None,
                }],
            );

            let mut stack_index = tokenizer.stack.len();

            while stack_index > 0 {
                stack_index -= 1;

                if tokenizer.stack[stack_index] == *token_type {
                    break;
                }
            }

            assert_eq!(
                tokenizer.stack[stack_index], *token_type,
                "expected token type"
            );
            tokenizer.stack.remove(stack_index);

            index += 1;
        }
    }
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
    println!("flow_start");
    let next = info.next;
    info.next = Box::new(flow); // This is weird but Rust needs a function there.

    let size = info.continued;
    exit_containers(tokenizer, &mut info, size);

    tokenizer.go_until(next, eof_eol, move |(state, remainder)| {
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

    // To do: blank lines? Other things?
    if tokenizer.events.len() > 2
        && tokenizer.events[tokenizer.events.len() - 1].token_type == Token::LineEnding
    {
        info.last_line_ending_index = Some(tokenizer.events.len() - 2);
    } else {
        info.last_line_ending_index = None;
    }

    match result {
        State::Ok => {
            println!("State::Ok");
            exit_containers(tokenizer, &mut info, 0);
            tokenizer.events = info.map.consume(&mut tokenizer.events);
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

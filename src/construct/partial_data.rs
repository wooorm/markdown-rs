//! Data occurs in [text][] and [string][].
//!
//! It can include anything (including line endings), and stops at certain
//! characters.
//!
//! [string]: crate::content::string
//! [text]: crate::content::text

use crate::token::Token;
use crate::tokenizer::{Code, Event, EventType, State, StateFnResult, Tokenizer};
use crate::util::edit_map::EditMap;

/// At the beginning of data.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code, stop: Vec<Code>) -> StateFnResult {
    if stop.contains(&code) {
        tokenizer.enter(Token::Data);
        tokenizer.consume(code);
        (State::Fn(Box::new(|t, c| data(t, c, stop))), None)
    } else {
        at_break(tokenizer, code, stop)
    }
}

/// Before something.
///
/// ```markdown
/// > | abc
///     ^
/// ```
fn at_break(tokenizer: &mut Tokenizer, code: Code, stop: Vec<Code>) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(Token::LineEnding);
            (State::Fn(Box::new(|t, c| at_break(t, c, stop))), None)
        }
        _ if stop.contains(&code) => {
            tokenizer.register_resolver("data".to_string(), Box::new(resolve));
            (State::Ok, Some(vec![code]))
        }
        _ => {
            tokenizer.enter(Token::Data);
            data(tokenizer, code, stop)
        }
    }
}

/// In data.
///
/// ```markdown
/// > | abc
///     ^^^
/// ```
fn data(tokenizer: &mut Tokenizer, code: Code, stop: Vec<Code>) -> StateFnResult {
    let done = match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => true,
        _ if stop.contains(&code) => true,
        _ => false,
    };

    if done {
        tokenizer.exit(Token::Data);
        at_break(tokenizer, code, stop)
    } else {
        tokenizer.consume(code);
        (State::Fn(Box::new(|t, c| data(t, c, stop))), None)
    }
}

/// Merge adjacent data events.
pub fn resolve(tokenizer: &mut Tokenizer) -> Vec<Event> {
    let mut edit_map = EditMap::new();
    let len = tokenizer.events.len();
    let mut index = 0;

    // Loop through events and merge adjacent data events.
    while index < len {
        let event = &tokenizer.events[index];

        if event.event_type == EventType::Enter && event.token_type == Token::Data {
            let exit_index = index + 1;
            let mut exit_far_index = exit_index;

            // Find multiple `data` events.
            while exit_far_index + 1 < len
                && tokenizer.events[exit_far_index + 1].token_type == Token::Data
            {
                exit_far_index += 2;
            }

            if exit_far_index > exit_index {
                edit_map.add(exit_index, exit_far_index - exit_index, vec![]);

                // Change positional info.
                let exit_far = &tokenizer.events[exit_far_index];
                let point_end = exit_far.point.clone();
                let index_end = exit_far.index;
                let exit = &mut tokenizer.events[exit_index];
                exit.point = point_end;
                exit.index = index_end;
                index = exit_far_index;

                continue;
            }
        }

        index += 1;
    }

    edit_map.consume(&mut tokenizer.events)
}

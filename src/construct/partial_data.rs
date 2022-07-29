//! Data occurs in [text][] and [string][].
//!
//! It can include anything (including line endings), and stops at certain
//! characters.
//!
//! [string]: crate::content::string
//! [text]: crate::content::text

use crate::token::Token;
use crate::tokenizer::{EventType, State, Tokenizer};

/// At the beginning of data.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer, stop: &'static [u8]) -> State {
    match tokenizer.current {
        Some(byte) if stop.contains(&byte) => {
            tokenizer.enter(Token::Data);
            tokenizer.consume();
            State::Fn(Box::new(move |t| data(t, stop)))
        }
        _ => at_break(tokenizer, stop),
    }
}

/// Before something.
///
/// ```markdown
/// > | abc
///     ^
/// ```
fn at_break(tokenizer: &mut Tokenizer, stop: &'static [u8]) -> State {
    match tokenizer.current {
        None => State::Ok,
        Some(b'\n') => {
            tokenizer.enter(Token::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Token::LineEnding);
            State::Fn(Box::new(move |t| at_break(t, stop)))
        }
        Some(byte) if stop.contains(&byte) => {
            tokenizer.register_resolver_before("data".to_string(), Box::new(resolve_data));
            State::Ok
        }
        _ => {
            tokenizer.enter(Token::Data);
            data(tokenizer, stop)
        }
    }
}

/// In data.
///
/// ```markdown
/// > | abc
///     ^^^
/// ```
fn data(tokenizer: &mut Tokenizer, stop: &'static [u8]) -> State {
    let done = match tokenizer.current {
        None | Some(b'\n') => true,
        Some(byte) if stop.contains(&byte) => true,
        _ => false,
    };

    if done {
        tokenizer.exit(Token::Data);
        at_break(tokenizer, stop)
    } else {
        tokenizer.consume();
        State::Fn(Box::new(move |t| data(t, stop)))
    }
}

/// Merge adjacent data events.
pub fn resolve_data(tokenizer: &mut Tokenizer) {
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
                tokenizer
                    .map
                    .add(exit_index, exit_far_index - exit_index, vec![]);

                // Change positional info.
                let exit_far = &tokenizer.events[exit_far_index];
                tokenizer.events[exit_index].point = exit_far.point.clone();
                index = exit_far_index;

                continue;
            }
        }

        index += 1;
    }
}

//! Data occurs in [text][] and [string][].
//!
//! It can include anything (including line endings), and stops at certain
//! characters.
//!
//! [string]: crate::content::string
//! [text]: crate::content::text

use crate::event::{Kind, Name};
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// At beginning of data.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Make sure to eat the first `markers`.
        Some(byte) if tokenizer.tokenize_state.markers.contains(&byte) => {
            tokenizer.enter(Name::Data);
            tokenizer.consume();
            State::Next(StateName::DataInside)
        }
        _ => State::Retry(StateName::DataAtBreak),
    }
}

/// Before something.
///
/// ```markdown
/// > | abc
///     ^
/// ```
pub fn at_break(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Ok,
        Some(b'\n') => {
            tokenizer.enter(Name::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Name::LineEnding);
            State::Next(StateName::DataAtBreak)
        }
        Some(byte) if tokenizer.tokenize_state.markers.contains(&byte) => {
            tokenizer.register_resolver_before(ResolveName::Data);
            State::Ok
        }
        _ => {
            tokenizer.enter(Name::Data);
            State::Retry(StateName::DataInside)
        }
    }
}

/// In data.
///
/// ```markdown
/// > | abc
///     ^^^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    let done = match tokenizer.current {
        None | Some(b'\n') => true,
        Some(byte) if tokenizer.tokenize_state.markers.contains(&byte) => true,
        _ => false,
    };

    if done {
        tokenizer.exit(Name::Data);
        State::Retry(StateName::DataAtBreak)
    } else {
        tokenizer.consume();
        State::Next(StateName::DataInside)
    }
}

/// Merge adjacent data events.
pub fn resolve(tokenizer: &mut Tokenizer) {
    let mut index = 0;

    // Loop through events and merge adjacent data events.
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];

        if event.kind == Kind::Enter && event.name == Name::Data {
            let exit_index = index + 1;
            let mut exit_far_index = exit_index;

            // Find multiple `data` events.
            while exit_far_index + 1 < tokenizer.events.len()
                && tokenizer.events[exit_far_index + 1].name == Name::Data
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

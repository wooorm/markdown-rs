//! Utilities to deal with lists of events.

use crate::tokenizer::{Event, TokenType};

/// Skip from `index`, optionally past `token_types`.
pub fn opt(events: &[Event], index: usize, token_types: &[TokenType]) -> usize {
    skip_opt_with_direction(events, index, token_types, true)
}

/// Skip from `index`, optionally past `token_types`, backwards.
pub fn opt_back(events: &[Event], index: usize, token_types: &[TokenType]) -> usize {
    skip_opt_with_direction(events, index, token_types, false)
}

/// Skip internals.
fn skip_opt_with_direction(
    events: &[Event],
    index: usize,
    token_types: &[TokenType],
    forward: bool,
) -> usize {
    let mut index = index;

    while index < events.len() {
        let current = &events[index].token_type;

        if !token_types.contains(current) {
            break;
        }

        // assert_eq!(events[index].event_type, EventType::Enter);
        index = if forward { index + 1 } else { index - 1 };

        loop {
            if events[index].token_type == *current {
                // assert_eq!(events[index].event_type, EventType::Exit);
                index = if forward { index + 1 } else { index - 1 };
                break;
            }

            index = if forward { index + 1 } else { index - 1 };
        }
    }

    index
}

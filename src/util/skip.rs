//! Utilities to deal with lists of events.

use crate::token::Token;
use crate::tokenizer::{Event, EventType};

/// Skip from `index`, optionally past `token_types`.
pub fn opt(events: &[Event], index: usize, token_types: &[Token]) -> usize {
    skip_opt_with_direction(events, index, token_types, true)
}

/// Skip from `index`, optionally past `token_types`, backwards.
pub fn opt_back(events: &[Event], index: usize, token_types: &[Token]) -> usize {
    skip_opt_with_direction(events, index, token_types, false)
}

pub fn to(events: &[Event], mut index: usize, token_types: &[Token]) -> usize {
    while index < events.len() {
        let current = &events[index].token_type;

        if token_types.contains(current) {
            break;
        }

        index += 1;
    }

    index
}

/// Skip internals.
fn skip_opt_with_direction(
    events: &[Event],
    mut index: usize,
    token_types: &[Token],
    forward: bool,
) -> usize {
    let mut balance = 0;
    let open = if forward {
        EventType::Enter
    } else {
        EventType::Exit
    };

    while index < events.len() {
        let current = &events[index].token_type;

        if !token_types.contains(current) || events[index].event_type != open {
            break;
        }

        index = if forward { index + 1 } else { index - 1 };
        balance += 1;

        loop {
            balance = if events[index].event_type == open {
                balance + 1
            } else {
                balance - 1
            };

            if events[index].token_type == *current && balance == 0 {
                println!(
                    "close:it! {:?} {:?} {:?}",
                    events[index].token_type, balance, index
                );
                index = if forward { index + 1 } else { index - 1 };
                println!("index:break: {:?}", index);
                break;
            }

            index = if forward { index + 1 } else { index - 1 };
            println!("index:loop: {:?}", index);
        }
    }

    index
}

//! Collect info for MDX.

use crate::event::{Event, Kind, Name};
use crate::util::slice::{Position, Slice};
use alloc::{string::String, vec, vec::Vec};

pub type Stop = (usize, usize);

#[derive(Debug)]
pub struct Result {
    pub value: String,
    pub stops: Vec<Stop>,
}

pub fn collect(
    events: &[Event],
    bytes: &[u8],
    from: usize,
    names: &[Name],
    stop: &[Name],
) -> Result {
    let mut result = Result {
        value: String::new(),
        stops: vec![],
    };
    let mut index = from;

    while index < events.len() {
        if events[index].kind == Kind::Enter {
            if names.contains(&events[index].name) {
                // Include virtual spaces, and assume void.
                let value = Slice::from_position(
                    bytes,
                    &Position {
                        start: &events[index].point,
                        end: &events[index + 1].point,
                    },
                )
                .serialize();
                result
                    .stops
                    .push((result.value.len(), events[index].point.index));
                result.value.push_str(&value);
            }
        } else if stop.contains(&events[index].name) {
            break;
        }

        index += 1;
    }

    result
}

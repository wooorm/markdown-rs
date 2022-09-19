//! Collect info for MDX.

use crate::event::{Kind, Name, Point};
use crate::tokenizer::Tokenizer;
use crate::util::slice::{Position, Slice};
use alloc::{string::String, vec, vec::Vec};

pub type Location<'a> = (usize, &'a Point);

pub struct Result<'a> {
    pub start: &'a Point,
    pub value: String,
    pub locations: Vec<Location<'a>>,
}

pub fn collect<'a>(tokenizer: &'a Tokenizer, from: usize, names: &[Name]) -> Result<'a> {
    let mut result = Result {
        start: &tokenizer.events[from].point,
        value: String::new(),
        locations: vec![],
    };
    let mut index = from;
    let mut acc = 0;

    while index < tokenizer.events.len() {
        if tokenizer.events[index].kind == Kind::Enter
            && names.contains(&tokenizer.events[index].name)
        {
            // Include virtual spaces.
            let value = Slice::from_position(
                tokenizer.parse_state.bytes,
                &Position {
                    start: &tokenizer.events[index].point,
                    end: &tokenizer.events[index + 1].point,
                },
            )
            .serialize();
            acc += value.len();
            result.locations.push((acc, &tokenizer.events[index].point));
            result.value.push_str(&value);
        }

        index += 1;
    }

    result
}

// Turn an index of `result.value` into a point in the whole document.
pub fn place_to_point(result: &Result, place: usize) -> Point {
    let mut index = 0;
    let mut point = result.start;
    let mut rest = place;

    while index < result.locations.len() {
        point = result.locations[index].1;

        if result.locations[index].0 > place {
            break;
        }

        rest = place - result.locations[index].0;
        index += 1;
    }

    let mut point = point.clone();
    point.column += rest;
    point.index += rest;
    point
}

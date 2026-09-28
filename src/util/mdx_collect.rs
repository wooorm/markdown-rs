//! Collect info for MDX.

use crate::event::{Event, Kind, Name};
use crate::tokenizer::Tokenizer;
use crate::util::slice::{Position, Slice};
use alloc::{boxed::Box, string::String, vec::Vec};

pub type Stop = (usize, usize);

#[derive(Debug, Default)]
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
    let mut result = Result::default();
    collect_more(events, bytes, from, names, stop, &mut result);
    result
}

/// Add what [`collect`][] would add from `from` to `result`; events before
/// `from` must not change between calls.
pub fn collect_more(
    events: &[Event],
    bytes: &[u8],
    from: usize,
    names: &[Name],
    stop: &[Name],
    result: &mut Result,
) -> usize {
    let mut index = from;

    while index < events.len() {
        if events[index].kind == Kind::Enter {
            if names.contains(&events[index].name) {
                // Include virtual spaces, and assume void.
                let slice = Slice::from_position(
                    bytes,
                    &Position {
                        start: &events[index].point,
                        end: &events[index + 1].point,
                    },
                );
                result
                    .stops
                    .push((result.value.len(), events[index].point.index));
                result
                    .value
                    .extend(core::iter::repeat(' ').take(slice.before));
                result.value.push_str(slice.as_str());
                result
                    .value
                    .extend(core::iter::repeat(' ').take(slice.after));
            }
        } else if stop.contains(&events[index].name) {
            break;
        }

        index += 1;
    }

    index
}

/// Start collecting the body of an MDX expression or ESM at `tokenize_state.start`.
pub fn reset_collect(tokenizer: &mut Tokenizer) {
    if let Some(collect) = &mut tokenizer.tokenize_state.mdx_collect {
        let (cursor, result) = &mut **collect;
        *cursor = tokenizer.tokenize_state.start;
        result.value.clear();
        result.stops.clear();
    }
}

/// Add the events of `names` since the last call.
pub fn collect_new(tokenizer: &mut Tokenizer, names: &[Name]) {
    let start = tokenizer.tokenize_state.start;
    let (cursor, result) = &mut **tokenizer
        .tokenize_state
        .mdx_collect
        .get_or_insert_with(|| Box::new((start, Result::default())));
    *cursor = collect_more(
        &tokenizer.events,
        tokenizer.parse_state.bytes,
        *cursor,
        names,
        &[],
        result,
    );
}

/// Body collected by the last call to `collect_new`.
pub fn collected<'a>(tokenizer: &'a Tokenizer) -> &'a Result {
    &tokenizer.tokenize_state.mdx_collect.as_ref().unwrap().1
}

//! Turn a string of markdown into events.
// To do: this should start with `containers`, when they’re done.
// To do: definitions and such will mean more data has to be passed around.
use crate::content::flow::flow;
use crate::tokenizer::{as_codes, Code, Event};

/// Turn a string of markdown into events.
/// Passes the codes back so the compiler can access the source.
pub fn parse(value: &str) -> (Vec<Event>, Vec<Code>) {
    let codes = as_codes(value);
    // To do: pass a reference to this around, and slices in the (back)feeding. Might be tough.
    let events = flow(codes.clone());
    (events, codes)
}

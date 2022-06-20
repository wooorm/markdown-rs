//! To do.

use crate::tokenizer::Event;

pub fn link(events: &mut [Event], index: usize) {
    events[index - 2].next = Some(index);
    events[index].previous = Some(index - 2);
}

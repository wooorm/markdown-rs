//! Helpers to deal with several changes in events, batching them together.
//!
//! Preferably, changes should be kept to a minumum.
//! Sometimes, it’s needed to change the list of events, because parsing can be
//! messy, and it helps to expose a cleaner interface of events to the compiler
//! and other users.
//! It can also help to merge many adjacent similar events.
//! And, in other cases, it’s needed to parse subcontent: pass some events
//! through another tokenizer and inject the result.

use crate::tokenizer::Event;
use std::collections::HashMap;

/// Shift `previous` and `next` links according to `jumps`.
///
/// This fixes links in case there are events removed or added between them.
fn shift_links(events: &mut [Event], jumps: &[(usize, isize)]) {
    let map = |before| {
        let mut jump_index = 0;
        let mut jump = 0;

        while jump_index < jumps.len() {
            if jumps[jump_index].0 > before {
                break;
            }

            jump = jumps[jump_index].1;
            jump_index += 1;
        }

        #[allow(clippy::pedantic)]
        let next_i = (before as isize) + jump;
        assert!(next_i >= 0, "cannot shift before `0`");
        #[allow(clippy::pedantic)]
        let next = next_i as usize;
        next
    };

    let mut index = 0;

    while index < events.len() {
        let event = &mut events[index];
        event.previous = event.previous.map(map);
        event.next = event.next.map(map);
        index += 1;
    }
}

/// Make it easy to insert and remove things while being performant and keeping
/// links in check.
pub struct EditMap {
    /// Whether this map was consumed already.
    consumed: bool,
    /// Record of changes.
    map: HashMap<usize, (usize, Vec<Event>)>,
}

impl EditMap {
    /// Create a new edit map.
    pub fn new() -> EditMap {
        EditMap {
            consumed: false,
            map: HashMap::new(),
        }
    }
    /// Create an edit: a remove and/or add at a certain place.
    pub fn add(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        add_impl(self, index, remove, add, false);
    }
    pub fn add_before(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        add_impl(self, index, remove, add, true);
    }
    /// Done, change the events.
    pub fn consume(&mut self, events: &mut [Event]) -> Vec<Event> {
        let mut indices: Vec<&usize> = self.map.keys().collect();
        let mut next_events: Vec<Event> = vec![];
        let mut start = 0;

        assert!(!self.consumed, "cannot consume after consuming");
        self.consumed = true;

        indices.sort_unstable();

        let mut jumps: Vec<(usize, isize)> = vec![];
        let mut index_into_indices = 0;
        let mut shift = 0;
        while index_into_indices < indices.len() {
            let index = *indices[index_into_indices];
            let edit = self.map.get(&index).unwrap();

            #[allow(clippy::pedantic)]
            let next = shift + (edit.1.len() as isize) - (edit.0 as isize);
            shift = next;
            jumps.push((index, shift));
            index_into_indices += 1;
        }

        let mut index_into_indices = 0;

        while index_into_indices < indices.len() {
            let index = *indices[index_into_indices];

            if start < index {
                let append = &mut events[start..index].to_vec();
                shift_links(append, &jumps);
                next_events.append(append);
            }

            let (remove, add) = self.map.get(&index).unwrap();

            if !add.is_empty() {
                let append = &mut add.clone();
                let mut index = 0;

                while index < append.len() {
                    let event = &mut append[index];
                    assert!(event.previous.is_none(), "to do?");
                    assert!(event.next.is_none(), "to do?");
                    index += 1;
                }

                next_events.append(append);
            }

            start = index + remove;
            index_into_indices += 1;
        }

        if start < events.len() {
            next_events.append(&mut events[start..].to_vec());
        }

        next_events
    }
}

/// To do.
fn add_impl(
    edit_map: &mut EditMap,
    index: usize,
    mut remove: usize,
    mut add: Vec<Event>,
    before: bool,
) {
    assert!(!edit_map.consumed, "cannot add after consuming");

    if let Some((curr_remove, mut curr_add)) = edit_map.map.remove(&index) {
        // To do: these might have to be split in several chunks instead
        // of one, if links in `curr_add` are supported.
        remove += curr_remove;
        if before {
            add.append(&mut curr_add);
        } else {
            curr_add.append(&mut add);
            add = curr_add;
        }
    }

    edit_map.map.insert(index, (remove, add));
}

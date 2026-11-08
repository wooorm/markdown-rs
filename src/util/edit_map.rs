//! Deal with several changes in events, batching them together.
//!
//! Preferably, changes should be kept to a minimum.
//! Sometimes, it’s needed to change the list of events, because parsing can be
//! messy, and it helps to expose a cleaner interface of events to the compiler
//! and other users.
//! It can also help to merge many adjacent similar events.
//! And, in other cases, it’s needed to parse subcontent: pass some events
//! through another tokenizer and inject the result.

use crate::event::Event;
#[cfg(debug_assertions)]
use crate::event::Link;
use alloc::{vec, vec::Vec};

/// Shift `previous` and `next` links according to `jumps`.
///
/// This fixes links in case there are events removed or added between them.
fn shift_links(events: &mut [Event], jumps: &[(usize, usize, usize)]) {
    let mut jump_index = 0;
    let mut index = 0;
    let mut add = 0;
    let mut rm = 0;

    while index < events.len() {
        let rm_curr = rm;

        while jump_index < jumps.len() && jumps[jump_index].0 <= index {
            add = jumps[jump_index].2;
            rm = jumps[jump_index].1;
            jump_index += 1;
        }

        // Ignore items that will be removed.
        if rm > rm_curr {
            index += rm - rm_curr;
        } else {
            if let Some(link) = &events[index].link {
                if let Some(next) = link.next {
                    events[next].link.as_mut().unwrap().previous = Some(index + add - rm);

                    while jump_index < jumps.len() && jumps[jump_index].0 <= next {
                        add = jumps[jump_index].2;
                        rm = jumps[jump_index].1;
                        jump_index += 1;
                    }

                    events[index].link.as_mut().unwrap().next = Some(next + add - rm);
                    index = next;
                    continue;
                }
            }

            index += 1;
        }
    }
}

/// Tracks a bunch of edits.
#[derive(Debug)]
pub struct EditMap {
    /// Record of changes.
    map: Vec<(usize, usize, Vec<Event>)>,
}

impl EditMap {
    /// Create a new edit map.
    pub fn new() -> EditMap {
        EditMap { map: vec![] }
    }
    /// Create an edit: a remove and/or add at a certain place.
    pub fn add(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        add_impl(self, index, remove, add, false);
    }
    /// Create an edit: but insert `add` before existing additions.
    pub fn add_before(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        add_impl(self, index, remove, add, true);
    }
    /// Done, change the events.
    ///
    /// Edits must be in bounds (`index + remove <= events.len()`) and must
    /// not overlap.
    /// Links must point forward to events that have a link.
    /// An event that is kept must not link to an event that is removed:
    /// `shift_links` cannot fix such a link, and would underflow.
    /// These are checked in debug builds.
    pub fn consume(&mut self, events: &mut Vec<Event>) {
        self.map
            .sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        if self.map.is_empty() {
            return;
        }

        #[cfg(debug_assertions)]
        check(events, &self.map);

        // Calculate jumps: where items in the current list move to.
        let mut jumps = Vec::with_capacity(self.map.len());
        let mut index = 0;
        let mut add_acc = 0;
        let mut remove_acc = 0;
        while index < self.map.len() {
            let (at, remove, add) = &self.map[index];
            remove_acc += remove;
            add_acc += add.len();
            jumps.push((*at, remove_acc, add_acc));
            index += 1;
        }

        shift_links(events, &jumps);

        let len_before = events.len();
        let mut index = self.map.len();
        let mut vecs = Vec::with_capacity(index * 2 + 1);
        while index > 0 {
            index -= 1;
            vecs.push(events.split_off(self.map[index].0 + self.map[index].1));
            vecs.push(self.map[index].2.split_off(0));
            events.truncate(self.map[index].0);
        }
        vecs.push(events.split_off(0));

        events.reserve(len_before + add_acc - remove_acc);

        while let Some(mut slice) = vecs.pop() {
            events.append(&mut slice);
        }

        self.map.truncate(0);
    }
}

/// Check that edits and links can be applied.
///
/// `map` must be sorted.
#[cfg(debug_assertions)]
fn check(events: &[Event], map: &[(usize, usize, Vec<Event>)]) {
    let mut index = 0;
    let mut end = 0;

    while index < map.len() {
        let (at, remove, _) = &map[index];
        assert!(*at >= end, "expected edits to not overlap");
        end = at + remove;
        assert!(end <= events.len(), "expected edits to be in bounds");
        index += 1;
    }

    // Whether the event at `position` is removed.
    let removed = |position: usize| match map.binary_search_by(|edit| edit.0.cmp(&position)) {
        Ok(index) => map[index].1 > 0,
        Err(0) => false,
        Err(index) => position < map[index - 1].0 + map[index - 1].1,
    };

    index = 0;

    while index < events.len() {
        if let Some(Link {
            next: Some(next), ..
        }) = &events[index].link
        {
            assert!(
                *next > index && *next < events.len() && events[*next].link.is_some(),
                "expected link to point forward to a linked event"
            );
            assert!(
                removed(index) || !removed(*next),
                "expected kept event to not link to a removed event"
            );
        }

        index += 1;
    }
}

/// Create an edit.
fn add_impl(edit_map: &mut EditMap, at: usize, remove: usize, mut add: Vec<Event>, before: bool) {
    let mut index = 0;

    if remove == 0 && add.is_empty() {
        return;
    }

    while index < edit_map.map.len() {
        if edit_map.map[index].0 == at {
            edit_map.map[index].1 += remove;

            if before {
                add.append(&mut edit_map.map[index].2);
                edit_map.map[index].2 = add;
            } else {
                edit_map.map[index].2.append(&mut add);
            }

            return;
        }

        index += 1;
    }

    edit_map.map.push((at, remove, add));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Content, Kind, Link, Name, Point};

    fn event(next: Option<usize>) -> Event {
        Event {
            kind: Kind::Enter,
            name: Name::Data,
            point: Point {
                line: 1,
                column: 1,
                index: 0,
                vs: 0,
            },
            link: Some(Link {
                previous: None,
                next,
                content: Content::Text,
            }),
        }
    }

    #[test]
    fn consume() {
        let mut events = vec![event(Some(1)), event(None), event(None)];
        let mut map = EditMap::new();
        map.add(2, 1, vec![]);
        map.consume(&mut events);
        assert_eq!(events.len(), 2, "should remove events");
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "expected kept event to not link to a removed event")]
    fn consume_kept_link_to_removed() {
        let mut events = vec![event(Some(2)), event(None), event(None)];
        let mut map = EditMap::new();
        map.add(1, 2, vec![]);
        map.consume(&mut events);
    }
}

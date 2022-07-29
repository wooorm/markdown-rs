//! Utilities to deal with characters.

use crate::constant::TAB_SIZE;
use crate::tokenizer::{Event, EventType, Point};

/// A range between two places.
#[derive(Debug)]
pub struct Position<'a> {
    pub start: &'a Point,
    pub end: &'a Point,
}

impl<'a> Position<'a> {
    /// Get a position from an exit event.
    ///
    /// Looks backwards for the corresponding `enter` event.
    /// This does not support nested events (such as lists in lists).
    ///
    /// ## Panics
    ///
    /// This function panics if an enter event is given.
    /// When `micromark` is used, this function never panics.
    pub fn from_exit_event(events: &'a [Event], index: usize) -> Position<'a> {
        let exit = &events[index];
        debug_assert_eq!(
            exit.event_type,
            EventType::Exit,
            "expected `from_exit_event` to be called on `exit` event"
        );
        let mut enter_index = index - 1;

        loop {
            let enter = &events[enter_index];
            if enter.event_type == EventType::Enter && enter.token_type == exit.token_type {
                return Position {
                    start: &enter.point,
                    end: &exit.point,
                };
            }

            enter_index -= 1;
        }
    }
}

/// Chars belonging to a range.
///
/// Includes information on virtual spaces before and after the chars.
#[derive(Debug)]
pub struct Slice<'a> {
    pub bytes: &'a [u8],
    pub before: usize,
    pub after: usize,
}

impl<'a> Slice<'a> {
    /// Get the slice belonging to a position.
    pub fn from_point(bytes: &'a [u8], point: &Point) -> Slice<'a> {
        let mut before = point.vs;
        let mut start = point.index;
        let end = if start < bytes.len() {
            start + 1
        } else {
            start
        };

        // If we have virtual spaces before, it means we are past the actual
        // character at that index, and those virtual spaces.
        if before > 0 {
            before = TAB_SIZE - before;
            start += 1;
        };

        Slice {
            bytes: if start < end { &bytes[start..end] } else { &[] },
            before,
            after: 0,
        }
    }

    /// Get the slice belonging to a position.
    pub fn from_position(bytes: &'a [u8], position: &Position) -> Slice<'a> {
        let mut before = position.start.vs;
        let mut after = position.end.vs;
        let mut start = position.start.index;
        let mut end = position.end.index;

        // If we have virtual spaces before, it means we are past the actual
        // character at that index, and those virtual spaces.
        if before > 0 {
            before = TAB_SIZE - before;
            start += 1;
        };

        // If we have virtual spaces after, it means that character is included,
        // and one less virtual space.
        if after > 0 {
            after -= 1;
            end += 1;
        }

        Slice {
            bytes: &bytes[start..end],
            before,
            after,
        }
    }

    /// To do.
    // To do: rename to `len`?
    pub fn size(&self) -> usize {
        self.bytes.len() + self.before + self.after
    }

    // To do:
    // When we have u8s, we could use: <https://doc.rust-lang.org/std/str/fn.from_utf8.html>
    // to implement an `as_str`.

    /// To do.
    pub fn head(&self) -> Option<u8> {
        if self.before > 0 {
            Some(b' ')
        } else if self.bytes.is_empty() {
            None
        } else {
            Some(self.bytes[0])
        }
    }

    /// To do.
    pub fn serialize(&self) -> String {
        let mut string = String::with_capacity(self.size());
        let mut index = self.before;
        while index > 0 {
            string.push(' ');
            index -= 1;
        }
        // To do: invalid UTF8?
        string.push_str(std::str::from_utf8(self.bytes).unwrap());
        index = self.after;
        while index > 0 {
            string.push(' ');
            index -= 1;
        }

        string
    }
}

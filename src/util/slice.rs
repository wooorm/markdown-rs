//! Deal with bytes.

use crate::event::{Event, Kind, Point};
use crate::util::{
    classify_character::{classify_opt, Kind as CharacterKind},
    constant::TAB_SIZE,
};
use alloc::string::String;
use core::str;

/// Get a [`char`][] right before `index` in bytes (`&[u8]`).
///
/// In most cases, markdown operates on ASCII bytes.
/// In a few cases, it is unicode aware, so we need to find an actual char.
pub fn char_before_index(bytes: &[u8], index: usize) -> Option<char> {
    let start = if index < 4 { 0 } else { index - 4 };
    String::from_utf8_lossy(&bytes[start..index]).chars().last()
}

/// Get a [`char`][] right at `index` in bytes (`&[u8]`).
///
/// In most cases, markdown operates on ASCII bytes.
/// In a few cases, it is unicode aware, so we need to find an actual char.
pub fn char_after_index(bytes: &[u8], index: usize) -> Option<char> {
    let end = if index + 4 > bytes.len() {
        bytes.len()
    } else {
        index + 4
    };
    String::from_utf8_lossy(&bytes[index..end]).chars().next()
}

/// Classify a byte (or `char`).
pub fn byte_to_kind(bytes: &[u8], index: usize) -> CharacterKind {
    if index == bytes.len() {
        CharacterKind::Whitespace
    } else {
        let byte = bytes[index];
        if byte.is_ascii_whitespace() {
            CharacterKind::Whitespace
        } else if byte.is_ascii_punctuation() {
            CharacterKind::Punctuation
        } else if byte.is_ascii_alphanumeric() {
            CharacterKind::Other
        } else {
            // Otherwise: seems to be an ASCII control, so it seems to be a
            // non-ASCII `char`.
            classify_opt(char_after_index(bytes, index))
        }
    }
}

/// A range between two points.
#[derive(Debug)]
pub struct Position<'a> {
    /// Start point.
    pub start: &'a Point,
    /// End point.
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
            exit.kind,
            Kind::Exit,
            "expected `from_exit_event` to be called on `exit` event"
        );
        let mut enter_index = index - 1;

        loop {
            let enter = &events[enter_index];
            if enter.kind == Kind::Enter && enter.name == exit.name {
                return Position {
                    start: &enter.point,
                    end: &exit.point,
                };
            }

            enter_index -= 1;
        }
    }

    /// Turn a position into indices.
    ///
    /// Indices are places in `bytes` where this position starts and ends.
    ///
    /// > 👉 **Note**: indices cannot represent virtual spaces.
    pub fn to_indices(&self) -> (usize, usize) {
        (self.start.index, self.end.index)
    }
}

/// Bytes belonging to a range.
///
/// Includes info on virtual spaces before and after the bytes.
#[derive(Debug)]
pub struct Slice<'a> {
    /// Bytes.
    pub bytes: &'a [u8],
    /// Number of virtual spaces before the bytes.
    pub before: usize,
    /// Number of virtual spaces after the bytes.
    pub after: usize,
}

impl<'a> Slice<'a> {
    /// Get a slice for a single point.
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

    /// Get a slice for a single index.
    ///
    /// > 👉 **Note**: indices cannot represent virtual spaces.
    pub fn from_index(bytes: &'a [u8], index: usize) -> Slice<'a> {
        Slice::from_indices(bytes, index, index + 1)
    }

    /// Get a slice for a position.
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

    /// Get a slice for two indices.
    ///
    /// > 👉 **Note**: indices cannot represent virtual spaces.
    pub fn from_indices(bytes: &'a [u8], start: usize, end: usize) -> Slice<'a> {
        Slice {
            bytes: &bytes[start..end],
            before: 0,
            after: 0,
        }
    }

    /// Get the size of this slice, including virtual spaces.
    pub fn len(&self) -> usize {
        self.bytes.len() + self.before + self.after
    }

    /// Get the first byte in this slice, representing a virtual space as a
    /// space.
    pub fn head(&self) -> Option<u8> {
        if self.before > 0 {
            Some(b' ')
        } else if self.bytes.is_empty() {
            None
        } else {
            Some(self.bytes[0])
        }
    }

    /// Turn the slice into a `&str`.
    ///
    /// > 👉 **Note**: cannot represent virtual spaces.
    pub fn as_str(&self) -> &str {
        str::from_utf8(self.bytes).unwrap()
    }

    /// Turn the slice into a `String`.
    ///
    /// Support virtual spaces.
    pub fn serialize(&self) -> String {
        let mut string = String::with_capacity(self.len());
        let mut index = self.before;
        while index > 0 {
            string.push(' ');
            index -= 1;
        }
        string.push_str(self.as_str());
        index = self.after;
        while index > 0 {
            string.push(' ');
            index -= 1;
        }

        string
    }
}

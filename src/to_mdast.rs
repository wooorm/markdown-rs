//! Turn events into a syntax tree.

// To do: example.

use crate::event::Event;
use crate::mdast;
use crate::Options;
use alloc::vec;

/// Turn events and bytes into a syntax tree.
pub fn compile(events: &[Event], _bytes: &[u8], _options: &Options) -> mdast::Root {
    mdast::Root {
        kind: mdast::Kind::Root,
        children: vec![],
        position: Some(mdast::Position {
            start: if events.is_empty() {
                create_point(1, 1, 0)
            } else {
                point_from_event(&events[0])
            },
            end: if events.is_empty() {
                create_point(1, 1, 0)
            } else {
                point_from_event(&events[events.len() - 1])
            },
        }),
    }
}

fn point_from_event(event: &Event) -> mdast::Point {
    create_point(event.point.line, event.point.column, event.point.index)
}

fn create_point(line: usize, column: usize, offset: usize) -> mdast::Point {
    mdast::Point {
        line,
        column,
        offset,
    }
}

extern crate swc_common;
use micromark::{
    mdast::Stop,
    unist::{Point, Position},
    Location,
};
use swc_common::{BytePos, Span, SyntaxContext, DUMMY_SP};
use swc_ecma_visit::{noop_visit_mut_type, VisitMut};

/// Turn a unist position, into an SWC span, of two byte positions.
///
/// > 👉 **Note**: SWC byte positions are offset by one: they are `0` when they
/// > are missing or incremented by `1` when valid.
pub fn position_to_span(position: Option<&Position>) -> Span {
    position.map_or(DUMMY_SP, |d| Span {
        lo: point_to_bytepos(&d.start),
        hi: point_to_bytepos(&d.end),
        ctxt: SyntaxContext::empty(),
    })
}

/// Turn an SWC span, of two byte positions, into a unist position.
///
/// This assumes the span comes from a fixed tree, or is a dummy.
///
/// > 👉 **Note**: SWC byte positions are offset by one: they are `0` when they
/// > are missing or incremented by `1` when valid.
pub fn span_to_position(span: &Span, location: Option<&Location>) -> Option<Position> {
    let lo = span.lo.0 as usize;
    let hi = span.hi.0 as usize;

    if lo > 0 && hi > 0 {
        if let Some(location) = location {
            if let Some(start) = location.to_point(lo - 1) {
                if let Some(end) = location.to_point(hi - 1) {
                    return Some(Position { start, end });
                }
            }
        }
    }

    None
}

/// Turn a unist point into an SWC byte position.
///
/// > 👉 **Note**: SWC byte positions are offset by one: they are `0` when they
/// > are missing or incremented by `1` when valid.
pub fn point_to_bytepos(point: &Point) -> BytePos {
    BytePos(point.offset as u32 + 1)
}

/// Turn an SWC byte position into a unist point.
///
/// This assumes the byte position comes from a fixed tree, or is a dummy.
///
/// > 👉 **Note**: SWC byte positions are offset by one: they are `0` when they
/// > are missing or incremented by `1` when valid.
pub fn bytepos_to_point(bytepos: &BytePos, location: Option<&Location>) -> Option<Point> {
    let pos = bytepos.0 as usize;

    if pos > 0 {
        if let Some(location) = location {
            return location.to_point(pos - 1);
        }
    }

    None
}

/// Prefix an error message with an optional point.
pub fn prefix_error_with_point(reason: String, point: Option<&Point>) -> String {
    if let Some(point) = point {
        format!("{}: {}", point_to_string(point), reason)
    } else {
        reason
    }
}

/// Serialize a unist position for humans.
pub fn position_to_string(position: &Position) -> String {
    format!(
        "{}-{}",
        point_to_string(&position.start),
        point_to_string(&position.end)
    )
}

/// Serialize a unist point for humans.
pub fn point_to_string(point: &Point) -> String {
    format!("{}:{}", point.line, point.column)
}

/// Visitor to fix SWC byte positions.
///
/// This assumes the byte position comes from an **unfixed** tree.
///
/// > 👉 **Note**: SWC byte positions are offset by one: they are `0` when they
/// > are missing or incremented by `1` when valid.
#[derive(Debug, Default, Clone)]
pub struct RewriteContext<'a> {
    pub prefix_len: usize,
    pub stops: &'a [Stop],
    pub location: Option<&'a Location>,
}

impl<'a> VisitMut for RewriteContext<'a> {
    noop_visit_mut_type!();

    // Rewrite spans.
    fn visit_mut_span(&mut self, span: &mut Span) {
        let mut result = DUMMY_SP;
        let lo_rel = span.lo.0 as usize;
        let hi_rel = span.hi.0 as usize;

        if lo_rel > self.prefix_len && hi_rel > self.prefix_len {
            if let Some(lo_abs) =
                Location::relative_to_absolute(self.stops, lo_rel - 1 - self.prefix_len)
            {
                if let Some(hi_abs) =
                    Location::relative_to_absolute(self.stops, hi_rel - 1 - self.prefix_len)
                {
                    result = Span {
                        lo: BytePos(lo_abs as u32 + 1),
                        hi: BytePos(hi_abs as u32 + 1),
                        ctxt: SyntaxContext::empty(),
                    };
                }
            }
        }

        *span = result;
    }
}

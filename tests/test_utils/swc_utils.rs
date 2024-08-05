//! Lots of helpers for dealing with SWC, particularly from unist, and for
//! building its ES AST.

use swc_core::common::{BytePos, Span, DUMMY_SP};
use swc_core::ecma::visit::{noop_visit_mut_type, VisitMut};

/// Visitor to fix SWC byte positions by removing a prefix.
///
/// > 👉 **Note**: SWC byte positions are offset by one: they are `0` when they
/// > are missing or incremented by `1` when valid.
#[derive(Debug, Default, Clone)]
pub struct RewritePrefixContext {
    /// Size of prefix considered outside this tree.
    pub prefix_len: u32,
}

impl VisitMut for RewritePrefixContext {
    noop_visit_mut_type!();

    /// Rewrite spans.
    fn visit_mut_span(&mut self, span: &mut Span) {
        let mut result = DUMMY_SP;
        if span.lo.0 > self.prefix_len && span.hi.0 > self.prefix_len {
            result = create_span(span.lo.0 - self.prefix_len, span.hi.0 - self.prefix_len);
        }

        *span = result;
    }
}

/// Generate a span.
pub fn create_span(lo: u32, hi: u32) -> Span {
    Span {
        lo: BytePos(lo),
        hi: BytePos(hi),
    }
}

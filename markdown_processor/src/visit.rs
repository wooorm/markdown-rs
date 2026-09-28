//! Walk trees.

use markdown::mdast;

/// Call `visitor` on `node` and its descendants, in preorder; a replaced
/// node's new children are visited.
pub fn visit_mut(node: &mut mdast::Node, visitor: &mut impl FnMut(&mut mdast::Node)) {
    visitor(node);
    if let Some(children) = node.children_mut() {
        for child in children {
            visit_mut(child, visitor);
        }
    }
}

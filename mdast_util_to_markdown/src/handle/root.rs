use alloc::string::String;
use markdown::mdast::{Node, Root};

use crate::{
    message::Message,
    state::{Info, State},
};

use super::Handle;

impl Handle for Root {
    fn handle(
        &self,
        state: &mut State,
        info: &Info,
        _parent: Option<&Node>,
        node: &Node,
    ) -> Result<String, Message> {
        let has_phrasing = self.children.iter().any(phrasing);
        if has_phrasing {
            state.container_phrasing(node, info)
        } else {
            state.container_flow(node)
        }
    }
}

fn phrasing(child: &Node) -> bool {
    matches!(
        *child,
        Node::Break(_)
            | Node::Emphasis(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::InlineCode(_)
            | Node::Link(_)
            | Node::LinkReference(_)
            | Node::Strong(_)
            | Node::Text(_)
    )
}

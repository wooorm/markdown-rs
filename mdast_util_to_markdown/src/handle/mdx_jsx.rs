//! JS equivalent: https://github.com/syntax-tree/mdast-util-to-markdown/blob/main/lib/handle/text.js

use super::Handle;
use crate::{
    construct_name::ConstructName,
    state::{Info, State},
};
use alloc::{string::String, vec::Vec};
use markdown::{
    mdast::{AttributeContent, MdxJsxFlowElement, Node},
    message::Message,
    unist::Position,
};

#[allow(dead_code)]
trait MdxJsx {
    fn children(&self) -> &Vec<Node>;
    fn name(&self) -> &Option<String>;
    fn attributes(&self) -> &Vec<AttributeContent>;
    fn position(&self) -> &Option<Position>;
}

impl MdxJsx for MdxJsxFlowElement {
    fn children(&self) -> &Vec<Node> {
        &self.children
    }

    fn name(&self) -> &Option<String> {
        &self.name
    }

    fn attributes(&self) -> &Vec<AttributeContent> {
        &self.attributes
    }

    fn position(&self) -> &Option<Position> {
        &self.position
    }
}

impl Handle for MdxJsxFlowElement {
    fn handle(
        &self,
        _state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<String, Message> {
        Ok(String::new())
    }
}

#[allow(dead_code)]
fn create_indent(depth: usize) -> String {
    "  ".repeat(depth)
}

#[allow(dead_code)]
fn infer_depth(state: &State) -> usize {
    let mut depth: usize = 0;

    for construct_name in state.stack.iter().rev() {
        if matches!(
            construct_name,
            ConstructName::Blockquote | ConstructName::ListItem
        ) {
            break;
        } else {
            depth += 1;
        }
    }

    depth
}

use alloc::string::String;
use markdown::mdast::{Blockquote, Node};

use crate::{
    construct_name::ConstructName,
    message::Message,
    state::{Info, State},
};

use super::Handle;

impl Handle for Blockquote {
    fn handle(
        &self,
        state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        node: &Node,
    ) -> Result<alloc::string::String, Message> {
        state.enter(ConstructName::Blockquote);
        let value = state.container_flow(node)?;
        let value = state.indent_lines(&value, map);
        Ok(value)
    }
}

fn map(line: &str, _index: usize, blank: bool) -> String {
    let marker = ">";
    let total_allocation = marker.len() + line.len() + 1;
    let mut result = String::with_capacity(total_allocation);
    result.push_str(marker);
    if !blank {
        let blank_str = " ";
        result.push_str(blank_str);
    }
    result.push_str(line);
    result
}

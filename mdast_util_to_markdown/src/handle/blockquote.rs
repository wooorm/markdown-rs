use alloc::string::String;
use markdown::mdast::{Blockquote, Node};

use crate::{
    construct_name::ConstructName,
    message::Message,
    state::{Info, State},
    util::indent_lines::indent_lines,
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
        let value = indent_lines(&state.container_flow(node)?, map);
        Ok(value)
    }
}

fn map(value: &str, _line: usize, blank: bool) -> String {
    let marker = ">";
    let total_allocation = marker.len() + value.len() + 1;
    let mut result = String::with_capacity(total_allocation);
    result.push_str(marker);
    if !blank {
        let blank_str = " ";
        result.push_str(blank_str);
    }
    result.push_str(value);
    result
}

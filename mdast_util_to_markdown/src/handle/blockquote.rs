use alloc::string::String;
use markdown::{
    mdast::{Blockquote, Node},
    message::Message,
};

use crate::{
    construct_name::ConstructName,
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
        state.exit();
        Ok(value)
    }
}

fn map(line: &str, _index: usize, blank: bool) -> String {
    let mut result = String::with_capacity(2 + line.len());
    let marker = ">";
    result.push_str(marker);
    if !blank {
        result.push_str(" ");
    }
    result.push_str(line);
    result
}

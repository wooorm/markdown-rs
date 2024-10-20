//! JS equivalent: https://github.com/syntax-tree/mdast-util-mdx-expression/blob/main/lib/index.js#L42

use super::Handle;
use crate::state::{Info, State};
use alloc::{format, string::String};
use markdown::{
    mdast::{MdxFlowExpression, MdxTextExpression, Node},
    message::Message,
};

impl Handle for MdxFlowExpression {
    fn handle(
        &self,
        state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<alloc::string::String, Message> {
        Ok(handle_mdx_expression(&self.value, state))
    }
}

impl Handle for MdxTextExpression {
    fn handle(
        &self,
        state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<alloc::string::String, Message> {
        Ok(handle_mdx_expression(&self.value, state))
    }
}

fn handle_mdx_expression(value: &str, state: &State) -> String {
    let result = state.indent_lines(value, |line, index, blank| {
        let space = if index == 0 || blank { "" } else { "  " };
        let mut results = String::with_capacity(space.len() + line.len());
        results.push_str(space);
        results.push_str(line);
        results
    });

    format!("{{{}}}", result)
}

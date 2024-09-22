use alloc::string::ToString;
use markdown::{
    mdast::{Break, Node},
    message::Message,
};

use crate::{
    state::{Info, State},
    util::pattern_in_scope::pattern_in_scope,
};

use super::Handle;

impl Handle for Break {
    fn handle(
        &self,
        state: &mut State,
        info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<alloc::string::String, Message> {
        for pattern in state.r#unsafe.iter() {
            if pattern.character == '\n' && pattern_in_scope(&state.stack, pattern) {
                let is_whitespace_or_tab = info.before.chars().any(|c| c == ' ' || c == '\t');
                if is_whitespace_or_tab {
                    return Ok("".to_string());
                }

                return Ok(" ".to_string());
            }
        }

        Ok("\\\n".to_string())
    }
}

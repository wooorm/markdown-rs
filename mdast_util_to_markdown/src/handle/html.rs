use markdown::mdast::{Html, Node};

use crate::{
    message::Message,
    state::{Info, State},
};

use super::Handle;

impl Handle for Html {
    fn handle(
        &self,
        _state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<alloc::string::String, Message> {
        Ok(self.value.clone())
    }
}

pub fn peek_html() -> char {
    '<'
}

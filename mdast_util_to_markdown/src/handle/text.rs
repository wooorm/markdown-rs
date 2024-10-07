use markdown::{
    mdast::{Node, Text},
    message::Message,
};

use crate::{
    state::{Info, State},
    util::safe::SafeConfig,
};

use super::Handle;

impl Handle for Text {
    fn handle(
        &self,
        state: &mut State,
        info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<alloc::string::String, Message> {
        Ok(state.safe(&self.value, &SafeConfig::new(info.before, info.after, None)))
    }
}

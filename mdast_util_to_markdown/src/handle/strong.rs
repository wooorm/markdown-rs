use alloc::format;
use markdown::{
    mdast::{Node, Strong},
    message::Message,
};

use crate::{
    construct_name::ConstructName,
    state::{Info, State},
    util::check_strong::check_strong,
};

use super::Handle;

impl Handle for Strong {
    fn handle(
        &self,
        state: &mut State,
        info: &Info,
        _parent: Option<&Node>,
        node: &Node,
    ) -> Result<alloc::string::String, Message> {
        let marker = check_strong(state)?;

        state.enter(ConstructName::Strong);

        let mut value = format!(
            "{}{}{}",
            marker,
            marker,
            state.container_phrasing(node, info)?
        );
        value.push(marker);
        value.push(marker);

        state.exit();

        Ok(value)
    }
}

pub fn peek_strong(state: &State) -> char {
    state.options.strong
}

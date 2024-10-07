use alloc::format;
use markdown::{
    mdast::{Emphasis, Node},
    message::Message,
};

use crate::{
    construct_name::ConstructName,
    state::{Info, State},
    util::check_emphasis::check_emphasis,
};

use super::Handle;

impl Handle for Emphasis {
    fn handle(
        &self,
        state: &mut State,
        info: &Info,
        _parent: Option<&Node>,
        node: &Node,
    ) -> Result<alloc::string::String, Message> {
        let marker = check_emphasis(state)?;

        state.enter(ConstructName::Emphasis);

        let mut value = format!("{}{}", marker, state.container_phrasing(node, info)?);
        value.push(marker);

        state.exit();

        Ok(value)
    }
}

pub fn peek_emphasis(state: &State) -> char {
    state.options.emphasis
}

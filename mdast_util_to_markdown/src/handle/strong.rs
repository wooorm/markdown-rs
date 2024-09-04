use alloc::format;
use markdown::mdast::Strong;

use crate::{
    construct_name::ConstructName,
    message::Message,
    state::{Info, State},
    util::check_strong::check_strong,
};

use super::Handle;

impl Handle for Strong {
    fn handle(&self, state: &mut State, info: &Info) -> Result<alloc::string::String, Message> {
        let marker = check_strong(state)?;

        state.enter(ConstructName::Strong);

        let mut value = format!(
            "{}{}{}",
            marker,
            marker,
            state.container_phrasing(self, info)?
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

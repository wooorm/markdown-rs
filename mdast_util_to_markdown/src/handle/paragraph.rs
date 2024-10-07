use markdown::{
    mdast::{Node, Paragraph},
    message::Message,
};

use crate::{
    construct_name::ConstructName,
    state::{Info, State},
};

use super::Handle;

impl Handle for Paragraph {
    fn handle(
        &self,
        state: &mut State,
        info: &Info,
        _parent: Option<&Node>,
        node: &Node,
    ) -> Result<alloc::string::String, Message> {
        state.enter(ConstructName::Paragraph);

        state.enter(ConstructName::Phrasing);
        let value = state.container_phrasing(node, info)?;
        // exit phrasing
        state.exit();
        // exit paragarph
        state.exit();
        Ok(value)
    }
}

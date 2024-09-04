use markdown::mdast::Paragraph;

use crate::{
    construct_name::ConstructName,
    message::Message,
    state::{Info, State},
};

use super::Handle;

impl Handle for Paragraph {
    fn handle(&self, state: &mut State, info: &Info) -> Result<alloc::string::String, Message> {
        state.enter(ConstructName::Paragraph);

        state.enter(ConstructName::Phrasing);
        let value = state.container_phrasing(self, info)?;
        // exit phrasing
        state.exit();
        // exit paragarph
        state.exit();
        Ok(value)
    }
}

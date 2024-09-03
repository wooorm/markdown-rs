use alloc::string::String;
use markdown::mdast::Paragraph;

use crate::{
    construct_name::ConstructName,
    state::{Info, State},
};

use super::Handle;

impl Handle for Paragraph {
    type Error = String;

    fn handle(&self, state: &mut State, info: &Info) -> Result<alloc::string::String, Self::Error> {
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

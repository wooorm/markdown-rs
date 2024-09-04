use markdown::mdast::Text;

use crate::{
    message::Message,
    state::{Info, State},
    util::safe::SafeConfig,
};

use super::Handle;

impl Handle for Text {
    fn handle(&self, state: &mut State, info: &Info) -> Result<alloc::string::String, Message> {
        Ok(state.safe(
            &self.value,
            &SafeConfig::new(Some(info.before), Some(info.after), None),
        ))
    }
}

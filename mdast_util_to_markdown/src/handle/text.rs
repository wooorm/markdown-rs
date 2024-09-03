use alloc::string::String;
use markdown::mdast::Text;

use crate::{
    state::{Info, State},
    util::safe::SafeConfig,
};

use super::Handle;

impl Handle for Text {
    type Error = String;

    fn handle(&self, state: &mut State, info: &Info) -> Result<alloc::string::String, Self::Error> {
        Ok(state.safe(
            &self.value,
            &SafeConfig::new(Some(info.before), Some(info.after), None),
        ))
    }
}

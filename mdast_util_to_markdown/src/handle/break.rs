use alloc::string::ToString;
use markdown::mdast::Break;
use regex::Regex;

use crate::{
    message::Message,
    state::{Info, State},
    util::pattern_in_scope::pattern_in_scope,
};

use super::Handle;

impl Handle for Break {
    fn handle(&self, state: &mut State, info: &Info) -> Result<alloc::string::String, Message> {
        for pattern in state.r#unsafe.iter() {
            if pattern.character == '\n' && pattern_in_scope(&state.stack, pattern) {
                let regex = Regex::new(r"[ \t]").unwrap();
                if regex.is_match(info.before) {
                    return Ok("".to_string());
                }

                return Ok(" ".to_string());
            }
        }

        Ok("\\\n".to_string())
    }
}

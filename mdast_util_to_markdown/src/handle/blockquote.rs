use alloc::string::String;
use markdown::mdast::BlockQuote;

use crate::{
    construct_name::ConstructName,
    message::Message,
    state::{Info, State},
    util::indent_lines::indent_lines,
};

use super::Handle;

impl Handle for BlockQuote {
    fn handle(&self, state: &mut State, _info: &Info) -> Result<alloc::string::String, Message> {
        state.enter(ConstructName::Blockquote);
        let value = indent_lines(&state.container_flow(self)?, map);
        Ok(value)
    }
}

fn map(value: &str, _line: usize, blank: bool) -> String {
    let mut result = String::from(">");
    if !blank {
        result.push(' ');
    }
    result.push_str(value);
    result
}

use alloc::format;
use markdown::mdast::{Node, ThematicBreak};

use crate::{
    message::Message,
    state::{Info, State},
    util::{check_rule::check_rule, check_rule_repetition::check_rule_repetition},
};

use super::Handle;

impl Handle for ThematicBreak {
    fn handle(
        &self,
        state: &mut State,
        _info: &Info,
        _parent: Option<&Node>,
        _node: &Node,
    ) -> Result<alloc::string::String, Message> {
        let marker = check_rule(state)?;
        let space = if state.options.rule_spaces { " " } else { "" };
        let mut value =
            format!("{}{}", marker, space).repeat(check_rule_repetition(state)? as usize);

        if state.options.rule_spaces {
            value.pop(); // remove the last space
            Ok(value)
        } else {
            Ok(value)
        }
    }
}

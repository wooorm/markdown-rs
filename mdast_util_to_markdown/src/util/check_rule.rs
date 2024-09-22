use alloc::{boxed::Box, format};
use markdown::message::Message;

use crate::state::State;

pub fn check_rule(state: &State) -> Result<char, Message> {
    let marker = state.options.rule;

    if marker != '*' && marker != '-' && marker != '_' {
        return Err(Message {
            reason: format!(
                "Cannot serialize rules with `{}` for `options.rule`, expected `*`, `-`, or `_`",
                marker
            ),
            rule_id: Box::new("unexpected-marker".into()),
            source: Box::new("mdast-util-to_markdown".into()),
            place: None,
        });
    }

    Ok(marker)
}

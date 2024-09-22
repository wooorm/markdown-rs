use alloc::{boxed::Box, format};
use markdown::message::Message;

use crate::state::State;

pub fn check_quote(state: &State) -> Result<char, Message> {
    let marker = state.options.quote;

    if marker != '"' && marker != '\'' {
        return Err(Message {
            reason: format!(
                "Cannot serialize title with `' {} '`  for `options.quote`, expected `\"`, or `'`",
                marker
            ),
            rule_id: Box::new("unexpected-marker".into()),
            source: Box::new("mdast-util-to_markdown".into()),
            place: None,
        });
    }

    Ok(marker)
}

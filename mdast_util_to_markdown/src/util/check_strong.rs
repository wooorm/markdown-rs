use alloc::{boxed::Box, format};
use markdown::message::Message;

use crate::state::State;

pub fn check_strong(state: &State) -> Result<char, Message> {
    let marker = state.options.strong;

    if marker != '*' && marker != '_' {
        return Err(Message {
            reason: format!(
                "Cannot serialize strong with `{}` for `options.strong`, expected `*`, or `_`",
                marker
            ),
            rule_id: Box::new("unexpected-marker".into()),
            source: Box::new("mdast-util-to-markdown".into()),
            place: None,
        });
    }

    Ok(marker)
}

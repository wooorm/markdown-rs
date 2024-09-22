use alloc::{boxed::Box, format};
use markdown::message::Message;

use crate::state::State;

pub fn check_emphasis(state: &State) -> Result<char, Message> {
    let marker = state.options.emphasis;

    if marker != '*' && marker != '_' {
        return Err(Message {
            reason: format!(
                "Cannot serialize emphasis with `{}` for `options.emphasis`, expected `*`, or `_`",
                marker
            ),
            rule_id: Box::new("unexpected-marker".into()),
            source: Box::new("mdast-util-to_markdown".into()),
            place: None,
        });
    }

    Ok(marker)
}

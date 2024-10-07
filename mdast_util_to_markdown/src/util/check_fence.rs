use alloc::{boxed::Box, format};
use markdown::message::Message;

use crate::state::State;

pub fn check_fence(state: &mut State) -> Result<char, Message> {
    let marker = state.options.fence;

    if marker != '`' && marker != '~' {
        return Err(Message {
            reason: format!(
                "Cannot serialize code with `{}` for `options.fence`, expected `` ` `` or `~`",
                marker
            ),
            rule_id: Box::new("unexpected-marker".into()),
            source: Box::new("mdast-util-to-markdown".into()),
            place: None,
        });
    }

    Ok(marker)
}

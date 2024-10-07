use alloc::{boxed::Box, format};
use markdown::message::Message;

use crate::state::State;

pub fn check_bullet(state: &mut State) -> Result<char, Message> {
    let marker = state.options.bullet;

    if marker != '*' && marker != '+' && marker != '-' {
        return Err(Message {
            reason: format!(
                "Cannot serialize items with `' {} '` for `options.bullet`, expected `*`, `+`, or `-`",
                marker
            ),
            rule_id: Box::new("unexpected-marker".into()),
            source: Box::new("mdast-util-to-markdown".into()),
            place: None,
        });
    }

    Ok(marker)
}

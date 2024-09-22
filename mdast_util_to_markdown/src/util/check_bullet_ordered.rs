use alloc::{boxed::Box, format};
use markdown::message::Message;

use crate::state::State;

pub fn check_bullet_ordered(state: &mut State) -> Result<char, Message> {
    let marker = state.options.bullet_ordered;

    if marker != '.' && marker != ')' {
        return Err(Message {
            reason: format!(
                "Cannot serialize items with `' {} '` for `options.bullet_ordered`, expected `.` or `)`",
                marker
            ),
            rule_id: Box::new("unexpected-marker".into()),
            source: Box::new("mdast-util-to_markdown".into()),
            place: None,
        });
    }

    Ok(marker)
}
